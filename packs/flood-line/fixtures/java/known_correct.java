// Test-only known-correct Java fixture for flood-line (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long trapped_water(long[] heights) {
        int left = 0, right = heights.length - 1;
        long leftBound = 0, rightBound = 0, water = 0;
        while (left < right) {
            if (heights[left] <= heights[right]) {
                leftBound = Math.max(leftBound, heights[left]);
                water += leftBound - heights[left];
                left += 1;
            } else {
                rightBound = Math.max(rightBound, heights[right]);
                water += rightBound - heights[right];
                right -= 1;
            }
        }
        return water;
    }
}
