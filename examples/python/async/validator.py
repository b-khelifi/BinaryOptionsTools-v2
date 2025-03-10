from BinaryOptionsToolsV2 import RawValidator

if __name__ == "__main__":
    none = RawValidator()
    regex = RawValidator.regex("([A-Z])\w+")
    start = RawValidator.starts_with("Hello")
    end = RawValidator.ends_with("Bye")
    contains = RawValidator.contains("World")
    rnot = RawValidator.ne(contains)

    # Modified for better testing - smaller groups with predictable outcomes
    rall = RawValidator.all([regex, start])  # Will need both capital letter and "Hello" at start
    rany = RawValidator.any([contains, end])  # Will need either "World" or end with "Bye"

    print(f"None validator: {none.check('hello')} (Expected: True)")
    print(f"Regex validator: {regex.check('Hello')} (Expected: True)")
    print(f"Regex validator: {regex.check('hello')} (Expected: False)")
    print(f"Starts_with validator: {start.check('Hello World')} (Expected: True)")
    print(f"Starts_with validator: {start.check('hi World')} (Expected: False)")
    print(f"Ends_with validator: {end.check('Hello Bye')} (Expected: True)")
    print(f"Ends_with validator: {end.check('Hello there')} (Expected: False)")
    print(f"Contains validator: {contains.check('Hello World')} (Expected: True)")
    print(f"Contains validator: {contains.check('Hello there')} (Expected: False)")
    print(f"Not validator: {rnot.check('Hello World')} (Expected: False)")
    print(f"Not validator: {rnot.check('Hello there')} (Expected: True)")

    # Testing the all validator
    print(f"All validator: {rall.check('Hello World')} (Expected: True)")  # Starts with "Hello" and has capital
    print(f"All validator: {rall.check('hello World')} (Expected: False)")  # No capital at start
    print(f"All validator: {rall.check('Hey there')} (Expected: False)")  # Has capital but doesn't start with "Hello"

    # Testing the any validator
    print(f"Any validator: {rany.check('Hello World')} (Expected: True)")  # Contains "World"
    print(f"Any validator: {rany.check('Hello Bye')} (Expected: True)")  # Ends with "Bye"
    print(f"Any validator: {rany.check('Hello there')} (Expected: False)")  # Neither contains "World" nor ends with "Bye"