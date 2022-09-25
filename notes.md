# Done

give users options on following symlinks
return project last edited??
print most recently modified time

# Current Activity


# Rest

return project "official name" from the package.json etc?
since we want to support multiple "project" types within the same directory,
  we need to return "directories", not "projects" since projects by definition only know 1 type
is there project specific data we need to pack in the enum, or could the enum be a bitfield :eyes:
path + bitfield combo is all thats needed to pass back?
should "extended" project info like last edited, size be calculated eagarly by the searcher, or lazily by the consumer?

clean up lots of unwrap, sub-par error handling

I'm happy with the ProjectType being an enum, but is there a nicer way with traits? will this harm the performance of the scan since
  we can't "inline" the logic?

can we multi-thread the kondo binary to "search ahead" and "clean behind" the user?
what are the tradeoffs?

tests ??? how does ProjectType not being trait impact tests?

works wonky on symlinks?

documentation on the library?

nicer output? coloured? tabbed?

progress ui?

performance comparison of dynamic trait logic to detect directory project type vs inlined/combined enum

walkdir cycle detection?
