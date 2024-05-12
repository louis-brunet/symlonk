CARGO = cargo
RUN = $(CARGO) run --


build:
	$(CARGO) build

schema:
	$(RUN) create schema | jq > docs/generated-schema.json

.PHONY: build schema
