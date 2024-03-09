#include <iostream>
#include <sstream>
#include <string>

int main() {
    int64_t index = 1;
    int64_t limit = 99999999; // Change 'limit' to the desired range

    std::ostringstream output_buffer;

    const int64_t page_threshold = 4096; // Adjust this threshold as needed

    for (int64_t i = index; i <= limit; i++) {
        bool isFizz = (i % 3 == 0);
        bool isBuzz = (i % 5 == 0);

        if (isFizz || isBuzz) {
            if (isFizz) output_buffer << "fizz";
            if (isBuzz) output_buffer << "buzz";
        } else {
            output_buffer << i;
        }

        output_buffer << '\n';

        // Check if the buffer size exceeds the threshold and flush it
        if (output_buffer.tellp() > page_threshold) {
            std::cout << output_buffer.str();
            output_buffer.str(""); // Clear the buffer
        }
    }

    // Flush the remaining content in the buffer
    std::cout << output_buffer.str();

    std::cout << "OK\n";

    return 0;
}

