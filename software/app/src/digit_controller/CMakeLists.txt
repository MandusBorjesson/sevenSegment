find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})

add_library(digit_controller
    src/digit.c
)

target_include_directories(digit_controller PUBLIC inc)

target_link_libraries(digit_controller PUBLIC zephyr_interface)
