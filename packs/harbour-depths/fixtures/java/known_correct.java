// Test-only known-correct Java fixture for harbour-depths (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long deepest_water(long[] heights) {
        int left = 0, right = heights.length - 1;
        long deepest = 0;
        while (left < right) {
            long held = Math.min(heights[left], heights[right]) * (right - left);
            if (held > deepest) deepest = held;
            if (heights[left] <= heights[right]) left += 1;
            else right -= 1;
        }
        return deepest;
    }
}
