// Test-only known-correct Java fixture for lit-order.
public class Solution {
    public static long[] lit_order(long[] marks, long[] nxt) {
        java.util.List<Long> out = new java.util.ArrayList<>();
        for (long i = nxt.length > 0 ? 0 : -1; i != -1; i = nxt[(int) i]) { out.add(marks[(int) i]); }
        return out.stream().mapToLong(Long::longValue).toArray();
    }
}
