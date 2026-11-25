// Test-only known-correct Java fixture for reorder-embers.
public class Solution {
    public static long[] reorder_embers(long[] marks, long[] nxt) {
        java.util.List<Long> order = new java.util.ArrayList<>();
        for (long i = nxt.length > 0 ? 0 : -1; i != -1; i = nxt[(int) i]) { order.add(marks[(int) i]); }
        int half = (order.size() + 1) / 2;
        java.util.List<Long> back = new java.util.ArrayList<>(order.subList(half, order.size()));
        java.util.Collections.reverse(back);
        java.util.List<Long> out = new java.util.ArrayList<>();
        for (int k = 0; k < half; k += 1) {
            out.add(order.get(k));
            if (k < back.size()) { out.add(back.get(k)); }
        }
        return out.stream().mapToLong(Long::longValue).toArray();
    }
}
