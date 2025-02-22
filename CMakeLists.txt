cmake_minimum_required(VERSION 3.12)

project(librespot VERSION 1.0.0 LANGUAGES C)

set(CMAKE_C_STANDARD 17)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)
set(CMAKE_STATIC_LIBRARY_PREFIX "")

# file(GLOB_RECURSE SOURCE_FILES CONFIGURE_DEPENDS
# src/${PROJECT_NAME}/*.c
# )
file(GLOB_RECURSE HEADER_FILES CONFIGURE_DEPENDS
    include/${PROJECT_NAME}/*.h
)
# file(GLOB_RECURSE PRIVATE_HEADER_FILES CONFIGURE_DEPENDS
# src/${PROJECT_NAME}/*.h
# )

add_library(${PROJECT_NAME} STATIC
    ${SOURCE_FILES}
    ${HEADER_FILES}
    ${PRIVATE_HEADER_FILES}
)
set_target_properties(${PROJECT_NAME} PROPERTIES LINKER_LANGUAGE C)

target_include_directories(${PROJECT_NAME} PUBLIC include)
set_target_properties(${PROJECT_NAME} PROPERTIES C_STANDARD 17 C_STANDARD_REQUIRED YES)

add_custom_target(librespot_rust_core
    COMMAND cargo build --release
    WORKING_DIRECTORY ${librespot_SOURCE_DIR}/core
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
    IMPORTED_LOCATION ${librespot_SOURCE_DIR}/core/target/release/${RUST_LIB_NAME}
)
add_dependencies(librespot_rust_lib librespot_rust_core)
target_link_libraries(${PROJECT_NAME} PRIVATE librespot_rust_lib)

install(TARGETS ${PROJECT_NAME}
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
    PUBLIC_HEADER DESTINATION include
)

install(FILES ${librespot_SOURCE_DIR}/core/target/release/${RUST_LIB_NAME}
    DESTINATION lib
)
