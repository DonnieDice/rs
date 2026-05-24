#!/usr/bin/env bash
set -euo pipefail

require_pattern() {
  local file="$1"
  local pattern="$2"
  local message="$3"

  if ! grep -Eq "$pattern" "$file"; then
    echo "sdk route regression check failed: $message" >&2
    echo "missing pattern '$pattern' in $file" >&2
    exit 1
  fi
}

client="crates/protondrive-api/src/client.rs"
config="crates/protondrive-api/src/config.rs"
nodes="crates/protondrive-api/src/endpoints/nodes.rs"
events="crates/protondrive-api/src/endpoints/events.rs"
shares="crates/protondrive-api/src/endpoints/shares.rs"

require_pattern "$config" "PROTON_API_BASE.*https://drive\\.proton\\.me/api" "metadata API base must stay on the official Drive API host"
require_pattern "$config" "DRIVE_SDK_VERSION" "SDK requests must carry an SDK version marker"
require_pattern "$config" "DEFAULT_TIMEOUT_MS.*30_000" "metadata timeout must mirror the official SDK"
require_pattern "$config" "DEFAULT_STORAGE_TIMEOUT_MS.*600_000" "storage timeout must mirror the official SDK"

require_pattern "$client" "x-pm-drive-sdk-version" "metadata and storage requests must include the Drive SDK version header"
require_pattern "$client" "application/vnd\\.protonmail\\.v1\\+json" "metadata requests must preserve the Proton v1 accept header"
require_pattern "$client" "pm-storage-token" "storage block requests must send the storage token header"
require_pattern "$client" "metadata_request_sets_official_headers_and_timeout" "metadata request header behavior must have unit coverage"
require_pattern "$client" "storage_upload_request_sets_official_headers_and_timeout" "storage request header behavior must have unit coverage"

require_pattern "$shares" "/drive/v2/shares/my-files" "my-files share lookup must use the current v2 route"
require_pattern "$nodes" "/drive/v2/volumes/.*/folders/.*/children" "folder children must use volume-scoped v2 routes"
require_pattern "$nodes" "/drive/v2/volumes/.*/links" "link metadata must use volume-scoped v2 routes"
require_pattern "$nodes" "/drive/v2/volumes/.*/links/.*/rename" "rename must use volume-scoped v2 routes"
require_pattern "$nodes" "/drive/v2/volumes/.*/links/.*/move" "move must use volume-scoped v2 routes"
require_pattern "$nodes" "/drive/v2/volumes/.*/trash_multiple" "trash must use volume-scoped v2 routes"
require_pattern "$nodes" "/drive/v2/volumes/.*/trash/delete_multiple" "trash deletion must use volume-scoped v2 routes"
require_pattern "$nodes" "/drive/v2/volumes/.*/remove-mine" "remove-mine must use volume-scoped v2 routes"
require_pattern "$nodes" "volume_node_route_builders_match_current_drive_api" "volume route construction must have unit coverage"

require_pattern "$events" "/core/v4/events/latest|/core/v5/events" "core event routes must be implemented before native session sync relies on SDK events"
require_pattern "$events" "/drive/volumes/.*/events/latest|/drive/v2/volumes/.*/events" "Drive event routes must be volume-scoped"

echo "SDK route regression checks passed."
