require 'xcodeproj'

project = Xcodeproj::Project.open('SCMessenger.xcodeproj')
target = project.targets.first

# Find the Transport group
transport_group = target.find_subpath('SCMessenger/Transport')

# Add the file
file_ref = transport_group.new_reference('SmartTransportRouter.swift')
file_ref.source_tree = 'SOURCE_ROOT'
file_ref.last_known_type = 'sourcecode.swift'

# Add to build phase
target.source_build_phase.add_file_reference(file_ref)

project.save
puts "File added successfully"
