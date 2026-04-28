#!/bin/bash
# Fix iOS Contact constructors to include lastKnownDeviceId parameter
set -e

file="iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift"

echo "Fixing iOS Contact constructors in $file..."

# Fix all instances where notes: is the last parameter and add lastKnownDeviceId
# Pattern 1: notes: existingContact.notes)
sed -i '' 's/\(notes: existingContact\.notes\)$/\1,/' "$file"

# Pattern 2: notes: updatedNotes)
sed -i '' 's/\(notes: updatedNotes\)$/\1,/' "$file"

# Pattern 3: notes: updatedNotesWithListeners)
sed -i '' 's/\(notes: updatedNotesWithListeners\)$/\1,/' "$file"

# Pattern 4: notes: routeNotes)
sed -i '' 's/\(notes: routeNotes\)$/\1,/' "$file"

# Pattern 5: notes: withListeners)
sed -i '' 's/\(notes: withListeners\)$/\1,/' "$file"

# Pattern 6: notes: notes) (for the case where notes variable is used)
sed -i '' 's/\(notes: notes\)$/\1,/' "$file"

# Now add lastKnownDeviceId: nil before closing parens on Contact constructors
# Find lines with trailing comma after notes and add lastKnownDeviceId on next line
sed -i '' '/notes: .*,$/a\
                    lastKnownDeviceId: nil
' "$file"

echo "✓ Fixed iOS Contact constructors"
