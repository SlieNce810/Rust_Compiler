func int main() {
    bool is_running;
    int led_state;
    int delay_counter;
    int delay_limit;
    int stm32_gpio_mode_reg;
    int stm32_gpio_output_reg;

    is_running = true;
    led_state = 0;
    delay_limit = 5000;

    stm32_gpio_mode_reg = 1;

    while (is_running) {
        if (led_state == 0) {
            led_state = 1;
        } else {
            led_state = 0;
        }

        stm32_gpio_output_reg = led_state;

        delay_counter = delay_limit;
        while (delay_counter > 0) {
            delay_counter = delay_counter - 1;
        }
    }

    return 0;
}
