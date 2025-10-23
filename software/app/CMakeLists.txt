cmake_minimum_required(VERSION 3.20.0)

list(APPEND BOARD_ROOT ${CMAKE_CURRENT_SOURCE_DIR})
find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})

add_subdirectory(src/digit_controller)

project(app)

target_sources(app PRIVATE src/main.c)

target_link_libraries(app PRIVATE
    digit_controller
)
