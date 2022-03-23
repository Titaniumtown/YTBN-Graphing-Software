## TODO:
1. Multiple functions in one graph.
	- Backend support
		- Integrals between functions (too hard to implement, maybe will shelve)
		- Display intersection between functions
		- Different colors
2. Rerwite of function parsing code
	- Non `y=` functions.
3. Smart display of graph
	- Display of intersections between functions
4. Properly parse lowercase `e` as euler's number
5. Allow constants in min/max integral input (like pi or euler's number)
6. Sliding values for functions (like a user-interactable slider that adjusts a variable in the function, like desmos)
7. nth derivative support (again)
8. Rewrite `FunctionEntry` to move more information and handling to egui_app (such as config changes)
9. Threading
10. Fix integral display
11. Improve loading indicator
	- Dynamically hide and unhide it when lagging
12. Add comments in `parsing::process_func_str` for it to be better understandable
13. Look into other, better methods of compression that would be faster
14. Better handling of panics and errors to display to the user
15. Display native/web dark mode detection as it's unneeded and can cause compat issues (see [egui #1305](https://github.com/emilk/egui/issues/1305))
16. Add tests for `SerdeValueHelper`
