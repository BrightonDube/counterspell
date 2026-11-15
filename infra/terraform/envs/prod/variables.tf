variable "name" {
  type        = string
  description = "Host name, for example counterspell-prod-1."
}

variable "datacentre" {
  type        = string
  description = "OVH datacentre code (EU only) [U]."
}

variable "plan" {
  type        = string
  description = "OVH VPS-2 plan code [U]."
}

variable "image" {
  type        = string
  description = "OS configuration value, for example Ubuntu 24.04 [U]."
}

variable "ssh_public_keys" {
  type        = list(string)
  description = "Public SSH keys only. Set in terraform.tfvars, which is never committed."
}

variable "ovh_subsidiary" {
  type        = string
  description = "OVH billing subsidiary [U]."
}

variable "vps_image_id" {
  type        = string
  description = "OS image UUID [U]."
}

variable "vps_duration" {
  type        = string
  description = "ISO 8601 subscription duration."
  default     = "P1M"
}

variable "vps_pricing_mode" {
  type        = string
  description = "Pricing mode that matches the duration."
  default     = "default"
}
