//! Expansion of a template into output tokens
//!
//! Contains the implementations of `fn expand()`
//! for the various template types in [`super::syntax`].

use super::framework::*;

impl<O> Expand<O> for SubstIf<O>
where
    Template<O>: ExpandInfallible<O>,
    O: ExpansionOutput,
{
    fn expand(&self, ctx: &Context, out: &mut O) -> syn::Result<()> {
        for (condition, consequence) in &self.tests {
            //dbg!(&condition);
            if condition.eval_bool(ctx)? {
                //dbg!(&consequence);
                consequence.expand(ctx, out);
                return Ok(());
            }
        }
        if let Some(consequence) = &self.otherwise {
            //dbg!(&consequence);
            consequence.expand(ctx, out);
        }
        Ok(())
    }
}

impl<O> SubstIf<O>
where
    Template<O>: ExpandInfallible<O>,
    O: ExpansionOutput,
{
    fn expand_select1(&self, ctx: &Context, out: &mut O) -> syn::Result<()> {
        let mut found: Result<Option<(Span, &Template<O>)>, Vec<ErrorLoc>> =
            Ok(None);

        for (condition, consequence) in &self.tests {
            if !condition.eval_bool(ctx)? {
                continue;
            }
            let cspan = condition.span();
            let error_loc = |span| (span, "true condition");
            match &mut found {
                Ok(None) => found = Ok(Some((cspan, consequence))),
                Ok(Some((span1, _))) => {
                    found = Err(vec![
                        ctx.error_loc(),
                        error_loc(*span1),
                        error_loc(cspan),
                    ])
                }
                Err(several) => several.push(error_loc(cspan)),
            }
        }
        let found = found
            .map_err(|several| several.error("multiple conditions matched"))?
            .map(|(_cspan, consequence)| consequence)
            .or(self.otherwise.as_deref())
            .ok_or_else(|| {
                [ctx.error_loc(), (self.kw_span, "select1 expansion")]
                    .error("no conditions matched, and no else clause")
            })?;
        found.expand(ctx, out);
        Ok(())
    }
}

impl SubstVType {
    fn expand(
        &self,
        ctx: &Context,
        out: &mut TokenAccumulator,
        kw_span: Span,
        self_def: SubstDetails<TokenAccumulator>,
    ) -> syn::Result<()> {
        let expand_spec_or_sd =
            |out: &mut _,
             spec: &Option<Argument<TokenAccumulator>>,
             sd: SubstDetails<TokenAccumulator>| {
                if let Some(spec) = spec {
                    spec.expand(ctx, out);
                    Ok(())
                } else {
                    sd.expand(ctx, out, kw_span)
                }
            };

        if !ctx.is_enum() {
            return expand_spec_or_sd(out, &self.self_, self_def);
        }
        // It's an enum.  We need to write the main type name,
        // and the variant.  Naively we might expect to just do
        //    TTYPE::VNAME
        // but that doesn't work, because if TTYPE has generics, that's
        //    TNAME::<TGENERICS>::VNAME
        // and this triggers bizarre (buggy) behaviour in rustc -
        // see rust-lang/rust/issues/108224.
        // So we need to emit
        //    TNAME::VNAME::<TGENERICS>
        //
        // The most convenient way to do that seems to be to re-parse
        // this bit of the expansion as a syn::Path.  That lets
        // us fish out the generics, for writing out later.

        let mut self_ty = TokenAccumulator::new();
        expand_spec_or_sd(&mut self_ty, &self.self_, self_def)?;
        let self_ty = self_ty.tokens()?;
        let mut self_ty: syn::Path =
            syn::parse2(self_ty).map_err(|mut e| {
                e.combine(kw_span.error(
                    "error re-parsing self type path for this expansion",
                ));
                e
            })?;

        let mut generics = mem::take(
            &mut self_ty
                .segments
                .last_mut()
                .ok_or_else(|| {
                    kw_span.error(
                        "self type path for this expansion is empty path!",
                    )
                })?
                .arguments,
        );

        out.append(self_ty);
        out.append(Token![::](kw_span));
        expand_spec_or_sd(out, &self.vname, SD::vname(Default::default()))?;
        let gen_content = match &mut generics {
            syn::PathArguments::AngleBracketed(content) => Some(content),
            syn::PathArguments::None => None,
            syn::PathArguments::Parenthesized(..) => {
                return Err([
                    (generics.span(), "generics"),
                    (kw_span, "template keyword"),
                ]
                .error("self type has parenthesised generics, not supported"))
            }
        };
        if let Some(gen_content) = gen_content {
            // Normalise `<GENERICS>` to `::<TGENERICS>`.
            gen_content
                .colon2_token
                .get_or_insert_with(|| Token![::](kw_span));
            out.append(&generics);
        }
        Ok(())
    }
}

impl SubstVPat {
    // $vpat      for struct    $tname         { $( $fname: $fpatname, ) }
    // $vpat      for enum      $tname::$vname { $( $fname: $fpatname, ) }
    fn expand(
        &self,
        ctx: &Context,
        out: &mut TokenAccumulator,
        kw_span: Span,
    ) -> syn::Result<()> {
        let self_def = SD::tname(Default::default());
        SubstVType::expand(&self.vtype, ctx, out, kw_span, self_def)?;

        let in_braces = braced_group(kw_span, |mut out| {
            WithinField::for_each(ctx, |ctx, field| {
                SD::fname::<TokenAccumulator>(())
                    .expand(ctx, &mut out, kw_span)?;
                out.append_tokens(&(), Token![:](kw_span))?;

                // Do the expansion with the paste machinery, since
                // that has a ready-made notion of what fprefix= might
                // allow, and how to use it.
                let mut paste = paste::Items::new(kw_span);
                if let Some(fprefix) = &self.fprefix {
                    fprefix.expand(ctx, &mut paste);
                } else {
                    paste.append_fixed_string("f_");
                }
                paste.append_identfrag_toks(&field.fname(kw_span))?;
                paste.assemble(out, None)?;
                out.append(Token![,](kw_span));

                Ok::<_, syn::Error>(())
            })
        })?;
        out.append(in_braces);
        Ok(())
    }
}

impl<O> ExpandInfallible<O> for Template<O>
where
    TemplateElement<O>: Expand<O>,
    O: ExpansionOutput,
{
    fn expand(&self, ctx_in: &Context, out: &mut O) {
        let mut ctx_buf;
        let mut definitions_here = vec![];
        let mut defconds_here = vec![];
        let mut ctx = ctx_in;

        for element in &self.elements {
            macro_rules! handle_definition { {
                $variant:ident, $store:expr
            } => {
                if let TE::Subst(Subst {
                    sd: SD::$variant(def, _),
                    ..
                }) = element
                {
                    // Doing this with a macro makes it nice and obvious
                    // to the borrow checker.
                    $store.push(def);
                    ctx_buf = ctx_in.clone();
                    ctx_buf.definitions.earlier = Some(&ctx_in.definitions);
                    ctx_buf.definitions.here = &definitions_here;
                    ctx_buf.definitions.conds = &defconds_here;
                    ctx = &ctx_buf;
                    continue;
                }

            } }

            handle_definition!(define, definitions_here);
            handle_definition!(defcond, defconds_here);

            let () = element
                .expand(ctx, out)
                .unwrap_or_else(|err| out.record_error(err));
        }
    }
}

impl Expand<TokenAccumulator> for TemplateElement<TokenAccumulator> {
    fn expand(
        &self,
        ctx: &Context,
        out: &mut TokenAccumulator,
    ) -> syn::Result<()> {
        match self {
            TE::Ident(tt) => out.append(tt.clone()),
            TE::Literal(tt, ..) => out.append(tt.clone()),
            TE::LitStr(tt) => out.append(tt.clone()),
            TE::Punct(tt, _) => out.append(tt.clone()),
            TE::Group {
                delim_span,
                delimiter,
                template,
                not_in_paste: _,
            } => {
                use proc_macro2::Group;
                let mut content = TokenAccumulator::new();
                template.expand(ctx, &mut content);
                let mut group = Group::new(*delimiter, content.tokens()?);
                group.set_span(*delim_span);
                out.append(TT::Group(group));
            }
            TE::Subst(exp) => {
                exp.expand(ctx, out)?;
            }
            TE::Repeat(repeated_template) => {
                repeated_template.expand(ctx, out);
            }
        }
        Ok(())
    }
}

impl<O> Expand<O> for Subst<O>
where
    O: ExpansionOutput,
    TemplateElement<O>: Expand<O>,
{
    fn expand(&self, ctx: &Context, out: &mut O) -> syn::Result<()> {
        self.sd.expand(ctx, out, self.kw_span)
    }
}

impl<O> SubstDetails<O>
where
    O: ExpansionOutput,
    TemplateElement<O>: Expand<O>,
{
    /// Expand this template element, by adding it to `O`
    ///
    /// This is done using `O`'s [`ExpansionOutput`] methods.
    fn expand(
        &self,
        ctx: &Context,
        out: &mut O,
        kw_span: Span,
    ) -> syn::Result<()> {
        // eprintln!("@@@@@@@@@@@@@@@@@@@@ EXPAND {:?}", self);

        let do_meta = |sm: &meta::SubstMeta<_>, out, meta| {
            sm.expand(ctx, kw_span, out, meta)
        };

        // Methods for handling generics.  Most take `composable: bool`,
        // which lets us control the trailing comma.  This is desirable
        // because we should include it for expansions like $tgens that the
        // user can append things to, but ideally *not* for expansions like
        // $ttype that the user can't.
        let do_tgnames = |out: &mut TokenAccumulator, composable| {
            for pair in ctx.top.generics.params.pairs() {
                use syn::GenericParam as GP;
                match pair.value() {
                    GP::Type(t) => out.append(&t.ident),
                    GP::Const(c) => out.append(&c.ident),
                    GP::Lifetime(l) => out.append(&l.lifetime),
                }
                out.append_maybe_punct_composable(&pair.punct(), composable);
            }
        };
        let do_tgens_nodefs = |out: &mut TokenAccumulator| {
            for pair in ctx.top.generics.params.pairs() {
                use syn::GenericParam as GP;
                let out_attrs = |out: &mut TokenAccumulator, attrs: &[_]| {
                    attrs.iter().for_each(|attr| out.append(attr));
                };
                match pair.value() {
                    GP::Type(t) => {
                        out_attrs(out, &t.attrs);
                        out.append(&t.ident);
                        out.append(&t.colon_token);
                        out.append(&t.bounds);
                    }
                    GP::Const(c) => {
                        out_attrs(out, &c.attrs);
                        out.append(&c.const_token);
                        out.append(&c.ident);
                        out.append(&c.colon_token);
                        out.append(&c.ty);
                    }
                    GP::Lifetime(l) => out.append(&l),
                }
                out.with_tokens(|out| {
                    pair.punct().to_tokens_punct_composable(out);
                });
            }
        };
        let do_tgens = |out: &mut TokenAccumulator, composable: bool| {
            out.append_maybe_punct_composable(
                &ctx.top.generics.params,
                composable,
            );
        };

        // There are three contexts where the top-level type
        // name might occur with generics, and two syntaxes:
        //   referring to the type    $ttype     Type::<G>
        //   impl'ing for the type    $ttype     Type::<G>
        //   defining a new type      $tdeftype  Type<G: bounds>
        // Handles $ttype and $tdeftype, and, indirectly, $vtype
        let do_ttype = |out: &mut O, colons: Option<()>, do_some_gens| {
            let _: &dyn Fn(&mut _, bool) = do_some_gens; // specify type
            let gens = &ctx.top.generics;
            let colons = gens
                .lt_token
                .and_then(|_| colons.map(|()| Token![::](kw_span)));
            out.append_idpath(
                kw_span,
                |_| {},
                &ctx.top.ident,
                |out| {
                    out.append(colons);
                    out.append(gens.lt_token);
                    do_some_gens(out, false);
                    out.append(gens.gt_token);
                },
                Grouping::Ungrouped,
            )
            .unwrap_or_else(|e| e.unreachable())
        };

        let do_maybe_delimited_group = |out, np, delim, content| {
            let _: &mut O = out;
            let _: &Template<TokenAccumulator> = content;
            out.append_tokens_with(np, |out| {
                if let Some(delim) = delim {
                    out.append(delimit_token_group(
                        delim,
                        kw_span,
                        |inside: &mut TokenAccumulator| {
                            Ok(content.expand(ctx, inside))
                        },
                    )?);
                } else {
                    content.expand(ctx, out);
                }
                Ok(())
            })
        };

        match self {
            SD::tname(_) => out.append_identfrag_toks(&ctx.top.ident)?,
            SD::ttype(_) => do_ttype(out, Some(()), &do_tgnames),
            SD::tdeftype(_) => do_ttype(out, None, &do_tgens),
            SD::vname(_) => {
                out.append_identfrag_toks(&ctx.syn_variant(&kw_span)?.ident)?
            }
            SD::fname(_) => {
                let fname = ctx.field(&kw_span)?.fname(kw_span);
                out.append_identfrag_toks(&fname)?;
            }
            SD::ftype(_) => {
                let f = ctx.field(&kw_span)?;
                out.append_syn_type(
                    kw_span,
                    f.field.ty.clone(),
                    Grouping::Invisible,
                );
            }
            SD::fpatname(_) => {
                let f = ctx.field(&kw_span)?;
                let fpatname =
                    Ident::new(&format!("f_{}", f.fname(kw_span)), kw_span);
                out.append_identfrag_toks(&fpatname)?;
            }
            SD::Xmeta(sm) => do_meta(sm, out, sm.pmetas(ctx, kw_span)?)?,
            SD::error(e, _) => e.throw(ctx)?,

            SD::Vis(vis, np) => {
                out.append_tokens(np, vis.syn_vis(ctx, kw_span)?)?
            }
            SD::tdefkwd(_) => {
                fn w<O>(out: &mut O, t: impl ToTokens)
                where
                    O: ExpansionOutput,
                {
                    out.append_identfrag_toks(&TokenPastesAsIdent(t))
                        .unwrap_or_else(|e| e.unreachable());
                }
                use syn::Data::*;
                match &ctx.top.data {
                    Struct(d) => w(out, &d.struct_token),
                    Enum(d) => w(out, &d.enum_token),
                    Union(d) => w(out, &d.union_token),
                };
            }

            SD::tattrs(ra, np, ..) => out.append_tokens_with(np, |out| {
                ra.expand(ctx, out, &ctx.top.attrs)
            })?,
            SD::vattrs(ra, np, ..) => out.append_tokens_with(np, |out| {
                let variant = ctx.variant(&kw_span)?.variant;
                let attrs = variant.as_ref().map(|v| &*v.attrs);
                ra.expand(ctx, out, attrs.unwrap_or_default())
            })?,
            SD::fattrs(ra, np, ..) => out.append_tokens_with(np, |out| {
                ra.expand(ctx, out, &ctx.field(&kw_span)?.field.attrs)
            })?,

            SD::tgens(np, ..) => out.append_tokens_with(np, |out| {
                do_tgens_nodefs(out);
                Ok(())
            })?,
            SD::tdefgens(np, ..) => out.append_tokens_with(np, |out| {
                do_tgens(out, true);
                Ok(())
            })?,
            SD::tgnames(np, ..) => out.append_tokens_with(np, |out| {
                do_tgnames(out, true);
                Ok(())
            })?,
            SD::twheres(np, ..) => out.append_tokens_with(np, |out| {
                if let Some(clause) = &ctx.top.generics.where_clause {
                    out.with_tokens(|out| {
                        clause.predicates.to_tokens_punct_composable(out);
                    });
                }
                Ok(())
            })?,

            SD::vpat(v, np, ..) => out.append_tokens_with(np, |out| {
                // This comment prevents rustfmt making this unlike the others
                v.expand(ctx, out, kw_span)
            })?,
            SD::vtype(v, np, ..) => out.append_tokens_with(np, |out| {
                v.expand(ctx, out, kw_span, SD::ttype(Default::default()))
            })?,

            SD::tdefvariants(content, np, ..) => {
                let delim = if ctx.is_enum() {
                    Some(Delimiter::Brace)
                } else {
                    None
                };
                do_maybe_delimited_group(out, np, delim, content)?;
            }
            SD::fdefine(spec_f, np, ..) => {
                out.append_tokens_with(np, |out| {
                    let field = ctx.field(&kw_span)?.field;
                    if let Some(driver_f) = &field.ident {
                        if let Some(spec_f) = spec_f {
                            spec_f.expand(ctx, out);
                        } else {
                            out.append(driver_f);
                        }
                    }
                    out.append(&field.colon_token);
                    Ok(())
                })?
            }
            SD::vdefbody(vname, content, np, ..) => {
                use syn::Fields as SF;
                let variant = ctx.variant(&kw_span)?;
                let struct_variant = variant.is_struct_toplevel_as_variant();
                if !struct_variant {
                    vname.expand(ctx, out);
                }
                let delim = match variant.fields {
                    SF::Unit => None,
                    SF::Unnamed(..) => Some(Delimiter::Parenthesis),
                    SF::Named(..) => Some(Delimiter::Brace),
                };
                do_maybe_delimited_group(out, np, delim, content)?;
                if !struct_variant {
                    // Any enum variant: terminate with a comma.
                    out.append_tokens(np, Token![,](kw_span))?;
                } else if matches!(variant.fields, SF::Named(_)) {
                    // struct {} at top-level: no terminator.
                } else {
                    // Unit or tuple struct: Terminate with a semicolon.
                    out.append_tokens(np, Token![;](kw_span))?;
                }
            }
            SD::Crate(np, ..) => out.append_tokens(np, &ctx.template_crate)?,

            SD::paste(content, ..) => {
                paste::expand(ctx, kw_span, content, out)?;
            }
            SD::ChangeCase(content, case, ..) => {
                let mut items = paste::Items::new(kw_span);
                content.expand(ctx, &mut items);
                items.assemble(out, Some(*case))?;
            }

            SD::define(..) | SD::defcond(..) => out.write_error(
                &kw_span,
                // I think this is impossible.  It could only occur if
                // someone parsed a Subst or SubstDetails that wasn't
                // in a Template.  It is Template.expand() that handles this.
                // We could possibly use proof tokens to see if this happens
                // and exclude it, but that would be super invasive.
                //
                // (There are some parallels between this and `${when }`)
                "${define } and ${defcond } only allowed in a full template",
            ),
            SD::UserDefined(name) => name.lookup_expand(ctx, out)?,

            SD::ignore(content, _) => {
                let mut ignore = O::new_with_span(kw_span);
                content.expand(ctx, &mut ignore);
                let () = ignore.ignore_impl()?;
            }
            SD::when(..) => out.write_error(
                &kw_span,
                "internal error - misplaced ${when } detected too late!",
            ),
            SD::If(conds, ..) => conds.expand(ctx, out)?,
            SD::select1(conds, ..) => conds.expand_select1(ctx, out)?,
            SD::For(repeat, _) => repeat.expand(ctx, out),

            SD::dbg(ddr) => ddr.expand(ctx, out, kw_span),
            SD::dbg_all_keywords(_) => dbg_allkw::dump(ctx),

            // ## maint/check-keywords-documented BoolOnly ##
            SD::is_struct(bo)
            | SD::is_enum(bo)
            | SD::is_union(bo)
            | SD::v_is_unit(bo)
            | SD::v_is_tuple(bo)
            | SD::v_is_named(bo)
            | SD::is_empty(bo, _)
            | SD::approx_equal(bo, _)
            | SD::False(bo)
            | SD::True(bo)
            | SD::not(_, bo)
            | SD::any(_, bo)
            | SD::all(_, bo) => out.append_bool_only(bo),
        };
        Ok(())
    }
}

impl<O: ExpansionOutput> DbgDumpRequest<O> {
    fn expand(&self, ctx: &Context, out: &mut O, kw_span: Span) {
        let desc =
            format!("derive-deftly dbg dump {}", self.display_heading(ctx),);

        let mut msg = String::new();
        let () = self.content_string;
        writeln!(
            msg, //
            r#"---------- {} expansion (start) ----------"#,
            desc,
        )
        .expect("write to String failed");

        out.dbg_expand(kw_span, ctx, &mut msg, &self.content_parsed);
        writeln!(
            msg,
            r#"
---------- {} expansion (end) ----------"#,
            desc
        )
        .expect("write to String failed");

        eprint!("{}", msg);
    }
}

impl ExplicitError {
    pub fn throw<T>(&self, ctx: &Context<'_>) -> Result<T, syn::Error> {
        Err([
            ctx.error_loc(), //
            (self.message.span(), "template"),
        ]
        .error(self.message.value()))
    }
}

impl DefinitionName {
    fn lookup_expand<O: ExpansionOutput>(
        &self,
        ctx: &Context<'_>,
        out: &mut O,
    ) -> syn::Result<()> {
        let (def, ctx) = ctx.find_definition(self)?.ok_or_else(|| {
            self.error(format!("user-defined expansion `{}` not found", self))
        })?;
        match &def.body {
            DefinitionBody::Paste(content) => {
                paste::expand(&ctx, def.body_span, content, out)?;
            }
            DefinitionBody::Normal(content) => {
                let not_in_paste =
                    O::not_in_paste(self).map_err(|mut unpasteable| {
                        unpasteable.combine(def.body_span.error(
 "user-defined expansion is not pasteable because it isn't, itself, ${paste }"
                        ));
                        unpasteable
                    })?;
                out.append_tokens_with(&not_in_paste, |out| {
                    content.expand(&ctx, out);
                    Ok(())
                })?;
            }
        }
        Ok(())
    }
}

impl RawAttr {
    fn expand(
        &self,
        ctx: &Context,
        out: &mut TokenAccumulator,
        attrs: &[syn::Attribute],
    ) -> syn::Result<()> {
        for attr in attrs {
            match self {
                RawAttr::Default => {
                    if ["deftly", "derive_deftly", "derive_deftly_adhoc"]
                        .iter()
                        .all(|exclude| !attr.path().is_ident(exclude))
                    {
                        out.append(attr);
                    }
                }
                RawAttr::Include { entries } => {
                    let ent = entries.iter().find(|ent| ent.matches(attr));
                    if let Some(ent) = ent {
                        ent.expand(ctx, out, attr)?;
                    }
                }
                RawAttr::Exclude { exclusions } => {
                    if !exclusions.iter().any(|excl| excl == attr.path()) {
                        out.append(attr);
                    }
                }
            }
        }
        Ok(())
    }
}

impl RawAttrEntry {
    fn matches(&self, attr: &syn::Attribute) -> bool {
        &self.path == attr.path()
    }

    fn expand(
        &self,
        _ctx: &Context,
        out: &mut TokenAccumulator,
        attr: &syn::Attribute,
    ) -> syn::Result<()> {
        out.append(attr);
        Ok(())
    }
}

impl<O> ExpandInfallible<O> for RepeatedTemplate<O>
where
    Template<O>: ExpandInfallible<O>,
    O: ExpansionOutput,
{
    fn expand(&self, ctx: &Context, out: &mut O) {
        // for_with_within expects a fallible closure, but we want to do
        // infallible work in our infallible context, so we use `Void`
        // as the error type and wrap each call in `Ok`.
        #[allow(clippy::unit_arg)] // clippy wants us to worsify the style
        match self.over {
            RO::Variants => ctx.for_with_within(|ctx, _: &WithinVariant| {
                Ok::<_, Void>(self.expand_inner(ctx, out))
            }),
            RO::Fields => ctx.for_with_within(|ctx, _: &WithinField| {
                Ok::<_, Void>(self.expand_inner(ctx, out))
            }),
        }
        .void_unwrap()
    }
}

impl<O: ExpansionOutput> RepeatedTemplate<O> {
    /// private, does the condition
    fn expand_inner(&self, ctx: &Context, out: &mut O)
    where
        Template<O>: ExpandInfallible<O>,
        O: ExpansionOutput,
    {
        let mut ctx = ctx.clone();
        ctx.within_loop = WithinLoop::When;

        for when in &self.whens {
            match when.eval_bool(&ctx) {
                Ok(true) => continue,
                Ok(false) => return,
                Err(e) => {
                    out.record_error(e);
                    return;
                }
            }
        }

        ctx.within_loop = WithinLoop::Body;
        self.template.expand(&ctx, out)
    }
}
