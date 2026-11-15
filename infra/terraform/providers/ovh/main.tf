# OVH implementation of the host contract.
#
# WARNING: `ovh_vps` is created through OVH's order workflow. `terraform apply` places a billable
# order. Plan and apply run only on Brighton's machine, and only at T23 (deploy-ops R1.6, R11.5).
# Nothing in this repository applies it.

resource "ovh_vps" "host" {
  display_name   = var.name
  ovh_subsidiary = var.ovh_subsidiary
  image_id       = var.vps_image_id
  public_ssh_key = var.ssh_public_keys[0]

  plan = [
    {
      duration     = var.vps_duration
      plan_code    = var.plan
      pricing_mode = var.vps_pricing_mode
      configuration = [
        {
          label = "vps_datacenter"
          value = var.datacentre
        },
        {
          label = "vps_os"
          value = var.image
        },
      ]
    }
  ]

  lifecycle {
    # Terminating the VPS is irreversible and ends the subscription. Removing it needs a deliberate
    # change to this line first.
    prevent_destroy = true
  }
}
