# Contract outputs. The ovh_vps resource does not export the public IPv4 address, so Brighton
# records it from the OVH control panel at T23. The output stays null on purpose; it is not guessed.

output "public_ipv4" {
  description = "Public IPv4 address. Null: the OVH resource does not export it. Recorded at T23."
  value       = null
}

output "ssh_port" {
  description = "SSH port on the host."
  value       = 22
}

output "vps_id" {
  description = "OVH service name of the VPS, used for imports and the T23 record."
  value       = ovh_vps.host.id
}
