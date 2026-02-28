# KnotenCore Standard Library (ASL)

The KnotenCore Standard Library (ASL) provides core utilities available to all Aether scripts via the module import system. These functions are written natively in Aether AST but optimized for general-purpose application development.

To include an ASL module in your script, use the `Import` node pointing to the relative `/stdlib/` path.

---

## 1. Array Utilities (`stdlib/array_utils.nod`)

A collection of tools for dynamically searching and manipulating Aether Array memory structures.

### `Array.Contains(arr, element)`
Iterates over an array to determine if a specific element exists.
*   **Parameters:**
    *   `arr` (Array): The target array.
    *   `element` (Any): The value to search for.
*   **Returns:** `Bool` (`true` if found, `false` otherwise).

### `Array.Max(arr)`
Scans a numerical array and returns the highest value.
*   **Parameters:**
    *   `arr` (Array of Int/Float): The target array.
*   **Returns:** `Int` or `Float` (Returns 0 if the array is empty).

### `Array.Reverse(arr)`
Creates and returns a new array with the elements in reverse order.
*   **Parameters:**
    *   `arr` (Array): The array to reverse.
*   **Returns:** `Array` (A newly allocated reversed array).

---

## 2. Advanced Mathematics (`stdlib/math_ext.nod`)

Extended mathematical functions built on top of the Aether execution primitives.

### `Math.Clamp(val, min, max)`
Restricts a value to be within a specified range.
*   **Parameters:**
    *   `val` (Int/Float): The value to clamp.
    *   `min` (Int/Float): The minimum allowable value.
    *   `max` (Int/Float): The maximum allowable value.
*   **Returns:** `Int` or `Float` (The clamped value).

### `Math.Lerp(a, b, t)`
Performs precise linear interpolation between two values (useful for animations and procedural transitions).
*   **Parameters:**
    *   `a` (Float): The start value.
    *   `b` (Float): The end value.
    *   `t` (Float): The interpolation factor (typically 0.0 to 1.0).
*   **Returns:** `Float` (`a + (b - a) * t`).

### `Math.DegToRad(deg)`
Converts degrees to radians.
*   **Parameters:**
    *   `deg` (Float): The angle in degrees.
*   **Returns:** `Float` (The angle in radians).

---

## 3. String Utilities (`stdlib/string_utils.nod`)

Basic string manipulation and checking utilities.

### `String.IsNotEmpty(str)`
Checks if a string (or an array) contains any elements.
*   **Parameters:**
    *   `str` (String): The text to evaluate.
*   **Returns:** `Bool` (`true` if length > 0, `false` otherwise).

### `String.FormatLog(msg)`
Prepends a standardized prefix to a message logging identifier.
*   **Parameters:**
    *   `msg` (String): The message to log.
*   **Returns:** `String` (e.g. `"[KnotenCore] Your message"`).
