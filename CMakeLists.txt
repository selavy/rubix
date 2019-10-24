cmake_minimum_required(VERSION 3.0)

project(rubix LANGUAGES C)

find_package(glfw3 3.3 REQUIRED)

add_executable(rubix main.c glad_gl.c)
target_link_libraries(rubix PUBLIC glfw)
set_target_properties(rubix PROPERTIES C_STANDARD 11)
target_include_directories(rubix PUBLIC include)
