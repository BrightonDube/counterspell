// Test-only known-correct Java fixture for reverse-order.
public class Solution {
    public static long[] reverse_order(long[] marks, long[] nxt) {
        java.util.List<Long> out = new java.util.ArrayList<>();
        for (long i = nxt.length > 0 ? 0 : -1; i != -1; i = nxt[(int) i]) { out.add(marks[(int) i]); }
        java.util.Collections.reverse(out);
        return out.stream().mapToLong(Long::longValue).toArray();
    }
}
