# Contract outputs. A provider module must replace these placeholders with its own values.
# The placeholders keep the contract valid on its own, so `validate` passes without a provider.

output "public_ipv4" {
  description = "Public IPv4 address of the host. Null until a provider fills it."
  value       = null
}

output "ssh_port" {
  description = "SSH port on the host. Port 22 is the default for the base role."
  value       = 22
}
