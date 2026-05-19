# Bot Deployment & Hydration Flow

This document details how `principia-aves` retrieves deployment files from Cloudflare R2 and prepares the localized configurations inside the target warmbot instance's directory.

## 1. R2 Hydration (Staging)

During an orchestration deployment request, keys matching the `OrchestrationR2Keys` definition are downloaded from the Cloudflare R2 bucket and saved into a shared local directory on disk:

| Key Field / Type | Source Path in R2 Bucket | Destination in Shared Stash |
| :--- | :--- | :--- |
| **Credentials Profile** | `bots/credentials/<profile_name>/` (Recursive Folder) | `bots/credentials/<profile_name>/` |
| **Script Configuration** | `bots/conf/scripts/<script_config_name>` (Single File) | `bots/conf/scripts/<script_config_name>` |
| **Controllers** | `bots/conf/controllers/<controller_config_name>` (List of files) | `bots/conf/controllers/<controller_config_name>` |
| **Scripts Runtime** | `bots/conf/scripts/runtime/` (Recursive Folder, optional) | `bots/conf/scripts/runtime/` |
| **Controllers Runtime** | `bots/conf/controllers/runtime/` (Recursive Folder, optional) | `bots/conf/controllers/runtime/` |

---

## 2. Local Copying & Prep (Stash to Warmbot Instance)

Once files are successfully hydrated, the sidecar copies the stash configurations directly into the specific target warmbot slot directory (`bots/instances/<warmbot_id>/conf/`):

### A. Credentials Profile
All files and folders under `bots/credentials/<profile_name>/` are copied recursively directly into:
```text
bots/instances/<warmbot_id>/conf/
```

> [!NOTE]
> Any subdirectories or files in the profile literally named `scripts` or `controllers` are skipped during this copy process to avoid clashing with strategy configs.

### B. Client Configuration (`conf_client.yml`)
If `conf_client.yml` was successfully copied from the credentials profile, the sidecar automatically overrides the `instance_id` field inside the YAML file to match the target warmbot name:
```yaml
instance_id: warmbot_1
```

### C. Script Configuration File
The strategy's script configuration YAML is copied into:
```text
bots/instances/<warmbot_id>/conf/scripts/<script_config_name>
```

### D. Controller Configurations
All requested controller YAML configuration files are copied into:
```text
bots/instances/<warmbot_id>/conf/controllers/<controller_config_name>
```
