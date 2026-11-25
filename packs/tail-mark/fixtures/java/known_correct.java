// Test-only known-correct Java fixture for tail-mark.
public class Solution {
    public static long tail_mark(long[] marks, long[] nxt) {
        long i = nxt.length > 0 ? 0 : -1, last = -1;
        while (i != -1) { last = i; i = nxt[(int) i]; }
        return last == -1 ? -1 : marks[(int) last];
    }
}
