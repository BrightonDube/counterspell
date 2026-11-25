// Test-only known-correct Java fixture for nth-tail.
public class Solution {
    public static long nth_tail(long[] marks, long[] nxt, long n) {
        if (nxt.length == 0) { return -1; }
        long lead = 0, trail = 0;
        for (long k = 0; k < n; k += 1) {
            if (lead == -1) { return -1; }
            lead = nxt[(int) lead];
        }
        if (lead == -1) { return -1; }
        while (nxt[(int) lead] != -1) { lead = nxt[(int) lead]; trail = nxt[(int) trail]; }
        return marks[(int) trail];
    }
}
