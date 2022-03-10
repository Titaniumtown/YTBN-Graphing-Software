## TODO:
1. Multiple functions in one graph.
    - Backend support
        - Integrals between functions (too hard to implement, maybe will shelve)
    - UI
        - Different colors (kinda)
2. Rerwite of function parsing code
    - Non `y=` functions.
3. Smart display of graph
    - Display of intersections between functions
    - Caching of roots and extrema
4. Fix integral line
5. re-add euler's number (well it works if you use capital e like `E^x`)
6. allow constants in min/max integral input (like pi or euler's number)
7. sliding values for functions (like a user-interactable slider that adjusts a variable in the function, like desmos)
8. Keybinds
9. nth derivative support (again)
10. Update function tests
11. rewrite FunctionEntry to move more information and handling to egui_app (such as config changes)
12. dynamically adjust newton iterations and resolution of graph based on performance and if the machine is lagging