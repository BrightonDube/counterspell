# Production host. The environment calls the OVH provider module. Switching provider means changing
# only the `source` line and writing a sibling directory under providers/.
#
# Plan and apply: Brighton only, at T23. See deploy/vps/ACCOUNTS.md for the record.

module "host" {
  source = "../../providers/ovh"

  name             = var.name
  datacentre       = var.datacentre
  plan             = var.plan
  image            = var.image
  ssh_public_keys  = var.ssh_public_keys
  ovh_subsidiary   = var.ovh_subsidiary
  vps_image_id     = var.vps_image_id
  vps_duration     = var.vps_duration
  vps_pricing_mode = var.vps_pricing_mode
}
