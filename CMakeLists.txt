cmake_minimum_required(VERSION 3.10)

project(librespot VERSION 1.0.0 LANGUAGES C)

set(CMAKE_C_STANDARD 17)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)
set(CMAKE_STATIC_LIBRARY_PREFIX "")

file(GLOB SOURCE_FILES src/${PROJECT_NAME}/*.c)
file(GLOB HEADER_FILES src/${PROJECT_NAME}/*.h)

add_library(${PROJECT_NAME} STATIC
    ${SOURCE_FILES}
    ${HEADER_FILES}
)

target_include_directories(${PROJECT_NAME} PUBLIC include include/librespot)
set_target_properties(${PROJECT_NAME} PROPERTIES C_STANDARD 17 C_STANDARD_REQUIRED YES)

add_custom_target(librespot_rust_core
    COMMAND cargo build --release
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}/_deps/${PROJECT_NAME}-src/core
)

if(WIN32)
    set(RUST_LIB_NAME "librespot.dll")
elseif(APPLE)
    set(RUST_LIB_NAME "librespot.dylib")
else()
    set(RUST_LIB_NAME "librespot.so")
endif()

add_library(librespot_rust_lib SHARED IMPORTED)
set_target_properties(librespot_rust_lib PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/core/target/release/${RUST_LIB_NAME}
)
add_dependencies(librespot_rust_lib librespot_rust_core)
target_link_libraries(${PROJECT_NAME} PRIVATE librespot_rust_lib)

install(TARGETS ${PROJECT_NAME}
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
    PUBLIC_HEADER DESTINATION inlcude
)

install(FILES ${CMAKE_SOURCE_DIR}/core/target/release/${RUST_LIB_NAME}
    DESTINATION lib
)

# make dev
add_library(${PROJECT_NAME}_dev STATIC
    ${SOURCE_FILES}
    ${HEADER_FILES}
)

add_custom_target(librespot_rust_core_dev
    COMMAND cargo build --release
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/core
)

add_library(librespot_rust_lib_dev SHARED IMPORTED)
set_target_properties(librespot_rust_lib_dev PROPERTIES
    IMPORTED_LOCATION ${CMAKE_BINARY_DIR}/_deps/${PROJECT_NAME}-src/core/target/release/${RUST_LIB_NAME}
)
add_dependencies(librespot_rust_lib_dev librespot_rust_core_dev)
target_link_libraries(${PROJECT_NAME}_dev PRIVATE librespot_rust_lib_dev)

add_custom_target(dev
    COMMAND ${CMAKE_COMMAND} --build . --target ${PROJECT_NAME}_dev
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Build for development (see if it compiles)"
)
