for file in *
    if test -f $file; and grep -q "\- links" $file
        mv $file topics
    end
end
