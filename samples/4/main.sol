use @std.types.int.Int
use @std.io.println

# open problem:
# in what kind of package would the "toString" function live?
# idea: in package of Type itself, and it is treated as typecast,
# e.g. calling it "String".
# That way it will get found automatically when resolving stuff.
# next question:
# Do I really want packages per directory?
# alternative: packages per file.
# possible problem: more subfolders
# possible benefit: flat hierarchies. 
# Though I think I actually _want_ to have int.add vs f32.add
fun main() = two

fun two() -> Int = 2
