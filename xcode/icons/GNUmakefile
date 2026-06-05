-:docs
	. make-icns.sh
.PHONY: docs
docs:
	@mkdir -p docs
	@./make-sizes.sh
	@./scripts/make-icns.sh
	@cp -R  ./docs/
	@cd banner && ./make-sizes.sh && cd ..
	@cp -R ./banner .
	@install -v README.md docs/index.md
	@sed 's/images/.\/images/' README.md > docs/index.md
