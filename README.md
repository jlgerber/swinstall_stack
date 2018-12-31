# swinstall_stack

This project implements a library and binary for interfacing with swinstalled software via the swinstall_stack xml file. Primarily, the focus is on supporting multiple schemas in the xml file in order to find the current swinstalled file within the swinstall_stack.

# Variations

There are two different implementations, exploring the use of dynamic vs static dispatch in the implememtation. To review the requirements:

We have an xml file whose structure we wish to evolve. In order to accomplish this, we have added a schema version attribute to the top level tag within the xml file. The workflow involves parsing this top level tag, identifying the schema of the xml file being parsed, and delegate responsibility to for further parsing to schema version specific implementor of a trait designed for that purpose.

- dynamic dispatch of code to parse schema version
- static dispatch of enum wrapped code to parse schema version