// Test-only known-correct Java fixture for next-lantern.
public class Solution {
    public static long[] next_taller(long[] heights) {
        int n = heights.length;
        long[] answer = new long[n];
        int[] waiting = new int[n];
        int top = 0;
        for (int index = n - 1; index >= 0; index -= 1) {
            while (top > 0 && heights[waiting[top - 1]] <= heights[index]) {
                top -= 1;
            }
            if (top > 0) {
                answer[index] = waiting[top - 1] - index;
            }
            waiting[top] = index;
            top += 1;
        }
        return answer;
    }
}
