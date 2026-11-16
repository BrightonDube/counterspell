# Provider-neutral inputs for one application host. Environments call a provider directory that
# implements this contract, and never name a provider resource. Only public values belong here
# (deploy-ops R1.3).
#
# The contract declares these inputs without using them. Provider directories consume them, so the
# unused-declaration rule is ignored here only.

# tflint-ignore: terraform_unused_declarations
variable "name" {
  type        = string
  description = "Host name, for example counterspell-prod-1."

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{2,62}$", var.name))
    error_message = "name must be 3 to 63 lower-case letters, digits or hyphens, starting with a letter."
  }
}

# tflint-ignore: terraform_unused_declarations
variable "datacentre" {
  type        = string
  description = "Provider region or datacentre code. EU only (system design section 11)."
}

# tflint-ignore: terraform_unused_declarations
variable "plan" {
  type        = string
  description = "Provider plan name. The provider maps it to its own catalogue entry."
}

# tflint-ignore: terraform_unused_declarations
variable "image" {
  type        = string
  description = "Operating system image name. Ubuntu LTS preferred (deploy-ops Q2)."
}

# tflint-ignore: terraform_unused_declarations
variable "ssh_public_keys" {
  type        = list(string)
  description = "Public SSH keys for the first login. Public values only; never a private key."

  validation {
    condition     = length(var.ssh_public_keys) > 0
    error_message = "At least one public SSH key is required so the host can be reached."
  }
}
