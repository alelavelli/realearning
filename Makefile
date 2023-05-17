format:
	cargo fmt
lint:
	cargo clippy

test:
	cargo test -- --test-threads=6

all: format lint test

tag:
	@# check if KIND variable is set
	@[ -z "$(KIND)" ] && echo KIND is empty && exit 1 || echo "creating tag $(KIND)"
	
	@# check if KIND variable has the allowed value
	@if [ "$${KIND}" != "major" -a "$${KIND}" != "minor" -a "$${KIND}" != "patch" -a "$${KIND}" != "beta" ]; then \
		echo "Error: KIND environment variable must be set to 'major', 'minor', 'patch' or 'beta'."; \
		exit 1; \
	fi

	@# read the current tag and export the three kinds
	$(eval CURRENT_TAG=$(shell git describe))
	$(eval MAJOR=$(word 1, $(subst ., , $(subst v, , $(subst -, , $(CURRENT_TAG))))))
	$(eval MINOR=$(word 2, $(subst ., , $(subst v, , $(subst -, , $(CURRENT_TAG))))))
	$(eval PATCH=$(word 3, $(subst ., , $(subst v, , $(subst -, , $(CURRENT_TAG))))))
	@echo "Version: $(CURRENT_TAG)"
	@echo "Major: $(MAJOR)"
	@echo "Minor: $(MINOR)"
	@echo "Patch: $(PATCH)"

	@# according to the kind set the new tag
	@# I know it's strange but if blocks must be written without indentation
ifeq ($(KIND),major)
	$(eval MAJOR := $(shell echo $(MAJOR) + 1 | bc))
	$(eval MINOR := 0)
	$(eval PATCH := 0)
else ifeq ($(KIND),minor)
	$(eval MINOR := $(shell echo $(MINOR) + 1 | bc))
	$(eval PATCH := 0)
else ifeq ($(KIND),patch)
	$(eval PATCH := $(shell echo $(PATCH) + 1 | bc))
else ifeq ($(KIND),beta)
	$(eval BETA := -beta)
endif
	
	# Set the new tag variable
	$(eval NEW_TAG=v$(MAJOR).$(MINOR).$(PATCH)$(BETA))
	$(eval MESSAGE=new version v$(MAJOR).$(MINOR).$(PATCH)$(BETA))
	@echo Creating new tag $(NEW_TAG)
	
	@# create new tag
	#git tag -a $(NEW_TAG) -m "$(MESSAGE)"

	@# push the tag
	#git push origin $(NEW_TAG)
