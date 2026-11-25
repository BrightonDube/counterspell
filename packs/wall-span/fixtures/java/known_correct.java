// Test-only known-correct Java fixture for wall-span.
public class Solution {
    public static long widest_fit(long[] wall) {
        long best = 0;
        int[] stack = new int[wall.length + 1];
        int top = 0;
        for (int index = 0; index <= wall.length; index += 1) {
            long height = index < wall.length ? wall[index] : 0;
            while (top > 0 && height < wall[stack[top - 1]]) {
                int popped = stack[top - 1];
                top -= 1;
                int left = top > 0 ? stack[top - 1] + 1 : 0;
                long area = wall[popped] * (index - left);
                if (area > best) {
                    best = area;
                }
            }
            stack[top] = index;
            top += 1;
        }
        return best;
    }
}
