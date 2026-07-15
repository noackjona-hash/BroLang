try
    # This should fail and jump to catch instead of crashing
    set file_data to read_file("does_not_exist.txt")
    print "This line should not print!"
catch
    print "Caught a file loading error successfully!"
end

print "Program completed execution cleanly."
