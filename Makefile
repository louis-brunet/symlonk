CARGO = cargo
RUN = $(CARGO) run --


build:
	$(CARGO) build --release

schema:
	$(RUN) create schema | jq > docs/generated-schema.json

.PHONY: build schema
