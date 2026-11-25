// Test-only known-correct Java fixture for tide-table (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long first_drop(long[] heights) {
        for (int index = 1; index < heights.length; index += 1) {
            if (heights[index] < heights[index - 1]) return index;
        }
        return -1;
    }
}
