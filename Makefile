.PHONY: publish

CRATE_NAME = gregtatum_symbol_table
VERSION = $(shell cargo pkgid | cut -d\# -f2)

publish-dry-run:
		@echo "Publishing $(VERSION) to crates.io..."
		cargo publish --dry-run

		@echo "Tagging release $(VERSION) in git..."
		@echo git tag $(VERSION)
		@echo git push --tags

		@echo "Done!"

publish:
		@echo "Publishing $(VERSION) to crates.io..."
		cargo publish

		@echo "Tagging release $(VERSION) in git..."
		git tag $(VERSION)
		git push --tags

		@echo "Done!"
