output "public_ipv4" {
  description = "Public IPv4. Null until Brighton records it from the OVH control panel (T23)."
  value       = module.host.public_ipv4
}

output "ssh_port" {
  description = "SSH port on the host."
  value       = module.host.ssh_port
}

output "vps_id" {
  description = "OVH service name of the VPS."
  value       = module.host.vps_id
}
