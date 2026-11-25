// Test-only known-correct Java fixture for rune-brackets.
public class Solution {
    public static long nested_runes(String inscription) {
        char[] openers = new char[inscription.length()];
        int top = 0;
        for (int index = 0; index < inscription.length(); index += 1) {
            char rune = inscription.charAt(index);
            if (rune == '(' || rune == '[' || rune == '{') {
                openers[top] = rune;
                top += 1;
            } else {
                if (top == 0) {
                    return 0;
                }
                char open = openers[top - 1];
                boolean matches = (open == '(' && rune == ')')
                    || (open == '[' && rune == ']')
                    || (open == '{' && rune == '}');
                if (!matches) {
                    return 0;
                }
                top -= 1;
            }
        }
        return top == 0 ? 1 : 0;
    }
}
