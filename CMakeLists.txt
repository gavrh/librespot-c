cmake_minimum_required(VERSION 3.10)

project(librespot VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_STATIC_LIBRARY_PREFIX "")

add_library(${PROJECT_NAME} STATIC
    src/librespot/player.cpp
    include/librespot/player.hpp
)

target_include_directories(${PROJECT_NAME} PUBLIC include include/librespot)

set_target_properties(${PROJECT_NAME} PROPERTIES CXX_STANDARD 17 CXX_STANDARD_REQUIRED YES)

add_custom_target(librespot_rust_build
    COMMAND cargo build --release
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/core
)

if(WIN32)
    set(RUST_LIB_NAME "librespot.dll")
elseif(APPLE)
    set(RUST_LIB_NAME "librespot.dylib")
else()
    set(RUST_LIB_NAME "librespot.so")
endif()

add_library(librespot_rustlib SHARED IMPORTED)
set_target_properties(librespot_rustlib PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/core/target/release/${RUST_LIB_NAME}
)
add_dependencies(librespot_rustlib librespot_rust_build)
target_link_libraries(${PROJECT_NAME} PRIVATE librespot_rustlib)

install(TARGETS ${PROJECT_NAME}
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
    PUBLIC_HEADER DESTINATION inlcude
)

install(FILES ${CMAKE_SOURCE_DIR}/core/target/release/${RUST_LIB_NAME}
    DESTINATION lib
)
