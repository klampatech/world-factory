#!/bin/bash
# Update issue status
ISSUE_ID="${1:-WOR-702}"
STATUS="${2:-done}"
COMMENT="${3:-}"

cd /home/kyle/projects/world-generator

PAPERCLIP_TASK_ID="$ISSUE_ID" \
PAPERCLIP_RUN_ID="${PAPERCLIP_RUN_ID:-test-$(date +%s)}" \
node -e "
const url = process.env.PAPERCLIP_API_URL + '/api/issues/' + process.env.PAPERCLIP_TASK_ID;
const body = { status: '$STATUS' };
if ('$COMMENT') body.comment = '$COMMENT';
fetch(url, {
  method: 'PATCH',
  headers: {
    'Authorization': 'Bearer ' + process.env.PAPERCLIP_API_KEY,
    'Content-Type': 'application/json',
    'X-Paperclip-Run-Id': process.env.PAPERCLIP_RUN_ID
  },
  body: JSON.stringify(body)
}).then(r => r.json()).then(d => console.log(JSON.stringify(d, null, 2)));
"