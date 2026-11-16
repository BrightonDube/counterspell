terraform {
  # The module names no provider. Each provider directory under infra/terraform/providers declares
  # its own provider requirement, so this contract never changes when the host provider changes.
  required_version = ">= 1.9.0"
}
