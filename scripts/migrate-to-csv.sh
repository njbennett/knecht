#!/bin/bash
set -e

# Migration script: Convert pipe-delimited .knecht/tasks to CSV format
# This script backs up the original file and converts it to CSV format

TASKS_FILE=".knecht/tasks"
BACKUP_FILE=".knecht/tasks.pipe-backup"

if [ ! -f "$TASKS_FILE" ]; then
    echo "No .knecht/tasks file found. Nothing to migrate."
    exit 0
fi

# Check if file is already in CSV format (contains comma after first character)
if head -n 1 "$TASKS_FILE" | grep -q '^[0-9]*,'; then
    echo "Tasks file appears to already be in CSV format. Skipping migration."
    exit 0
fi

echo "Migrating .knecht/tasks from pipe-delimited to CSV format..."

# Create backup
cp "$TASKS_FILE" "$BACKUP_FILE"
echo "Backup created at $BACKUP_FILE"

# Convert pipe-delimited to CSV using awk
awk -F'|' '{
    # Handle different field counts
    id = $1
    status = $2
    title = $3
    description = (NF >= 4) ? $4 : ""
    pain_count = (NF >= 5) ? $5 : ""
    
    # Escape quotes in title and description by doubling them
    gsub(/"/, "\"\"", title)
    gsub(/"/, "\"\"", description)
    
    # Unescape pipe-delimited escapes: \| -> | and \\ -> \
    gsub(/\\\|/, "|", title)
    gsub(/\\\\/, "\\", title)
    gsub(/\\\|/, "|", description)
    gsub(/\\\\/, "\\", description)
    
    # Output CSV format: id,status,"title","description",pain_count
    printf "%s,%s,\"%s\",\"%s\",%s\n", id, status, title, description, pain_count
}' "$BACKUP_FILE" > "$TASKS_FILE"

echo "Migration complete!"
echo "Original file backed up to: $BACKUP_FILE"
echo ""
echo "Please verify the migrated file looks correct:"
echo "  head .knecht/tasks"
echo ""
echo "If everything looks good, you can remove the backup:"
echo "  rm $BACKUP_FILE"