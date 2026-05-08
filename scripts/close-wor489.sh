#!/bin/bash
# Close WOR-489 via Paperclip API

curl -s -X PATCH "http://localhost:3000/api/issues/WOR-489" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer dev-key" \
  -H "X-Paperclip-Run-Id: $(date +%s)" \
  -d @- << 'EOF'
{
  "status": "done",
  "comment": "## WOR-489 Complete ✅\n\nWeb directory refactor confirmed complete:\n\n- web/css/styles.css (21,647 bytes)\n- web/js/api.js, app.js, dashboard.js, map-view.js, timeline.js\n- Both index.html and world.html load the modules\n- Build script copies css/ and js/ directories\n- npm run build succeeds\n\nNo blockers. Issue closed."
}
EOF
