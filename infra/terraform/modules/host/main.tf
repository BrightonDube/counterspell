# The contract has no resources of its own. A provider directory (for example providers/ovh) holds
# the resources, reads the inputs declared in variables.tf and fills the outputs in outputs.tf.
# Environments call a provider directory, never this contract directly.
