// Test-only known-correct Java fixture for rune-dial.
public class Solution {
    public static long[] run_median(long[] marks) {
        java.util.PriorityQueue<Long> lo =
            new java.util.PriorityQueue<>(java.util.Collections.reverseOrder());
        java.util.PriorityQueue<Long> hi = new java.util.PriorityQueue<>();
        long[] out = new long[marks.length];
        for (int i = 0; i < marks.length; i += 1) {
            lo.add(marks[i]);
            hi.add(lo.poll());
            if (hi.size() > lo.size()) {
                lo.add(hi.poll());
            }
            if (lo.size() == hi.size()) {
                out[i] = lo.peek() + hi.peek();
            } else {
                out[i] = 2 * lo.peek();
            }
        }
        return out;
    }
}
