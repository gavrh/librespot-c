cmake_minimum_required(VERSION 3.12)

project(librespot VERSION 1.0.0 LANGUAGES C)

include(CMakePackageConfigHelpers)
include(GNUInstallDirs)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)
set(CMAKE_STATIC_LIBRARY_PREFIX "")

file(GLOB HEADER_FILES CONFIGURE_DEPENDS
    include/${PROJECT_NAME}/*.h
)

set(EMPTY_SRC "${CMAKE_BINARY_DIR}/_.c")
file(WRITE ${EMPTY_SRC} "// auto-generated empty source\nvoid __empty(void) {}")

add_library(${PROJECT_NAME} STATIC ${EMPTY_SRC} ${HEADER_FILES})
set_target_properties(${PROJECT_NAME} PROPERTIES LINKER_LANGUAGE C)
target_include_directories(${PROJECT_NAME}
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)

if(WIN32)
    set(RUST_LIB_NAME "librespot_core.dll")
    set(RUST_LIB_INSTALL_DIR "${CMAKE_INSTALL_BINDIR}")
    if(MSVC)
        set(RUST_IMPORT_LIB_NAME "librespot_core.dll.lib")
    else()
        set(RUST_IMPORT_LIB_NAME "librespot_core.dll.a")
    endif()
elseif(APPLE)
    set(RUST_LIB_NAME "librespot_core.dylib")
    set(RUST_LIB_INSTALL_DIR "${CMAKE_INSTALL_LIBDIR}")
    set(RUST_IMPORT_LIB_NAME "")
else()
    set(RUST_LIB_NAME "librespot_core.so")
    set(RUST_LIB_INSTALL_DIR "${CMAKE_INSTALL_LIBDIR}")
    set(RUST_IMPORT_LIB_NAME "")
endif()
set(RUST_LIB_PATH "${CMAKE_CURRENT_SOURCE_DIR}/core/target/release/${RUST_LIB_NAME}")

add_library(librespot_core SHARED IMPORTED GLOBAL)
set_target_properties(librespot_core PROPERTIES
    IMPORTED_LOCATION "${CMAKE_BINARY_DIR}/lib/${RUST_LIB_NAME}"
)
if(WIN32)
    set_target_properties(librespot_core PROPERTIES
        IMPORTED_IMPLIB "${CMAKE_CURRENT_SOURCE_DIR}/core/target/release/${RUST_IMPORT_LIB_NAME}"
    )
endif()

add_custom_target(
    ${PROJECT_NAME}_rust
    COMMAND cargo build --release
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/core
)
add_custom_command(
    TARGET ${PROJECT_NAME}_rust
    POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E make_directory ${CMAKE_BINARY_DIR}/lib
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_LIB_PATH}
        ${CMAKE_BINARY_DIR}/lib/${RUST_LIB_NAME}
)

add_dependencies(${PROJECT_NAME} ${PROJECT_NAME}_rust)
target_link_libraries(${PROJECT_NAME} PUBLIC librespot_core)

# install librespot
install(
    TARGETS ${PROJECT_NAME}
    EXPORT ${PROJECT_NAME}Targets
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
)

install(
    FILES ${HEADER_FILES}
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}/${PROJECT_NAME}
)

install(
    EXPORT ${PROJECT_NAME}Targets
    FILE ${PROJECT_NAME}Targets.cmake
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}
)

set(LIBRESPOT_INSTALL_INCLUDEDIR "${CMAKE_INSTALL_INCLUDEDIR}")
set(LIBRESPOT_INSTALL_LIBDIR "${CMAKE_INSTALL_LIBDIR}")
set(LIBRESPOT_INSTALL_BINDIR "${CMAKE_INSTALL_BINDIR}")

configure_package_config_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/config.cmake.in
    ${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Config.cmake
    INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}
    PATH_VARS
        LIBRESPOT_INSTALL_INCLUDEDIR
        LIBRESPOT_INSTALL_LIBDIR
        LIBRESPOT_INSTALL_BINDIR
)

write_basic_package_version_file(
    ${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}ConfigVersion.cmake
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY SameMajorVersion
)

install(
    FILES
        ${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Config.cmake
        ${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}ConfigVersion.cmake
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}
)

# install rust core lib
install(
    FILES ${CMAKE_BINARY_DIR}/lib/${RUST_LIB_NAME}
    DESTINATION ${RUST_LIB_INSTALL_DIR}
)
if(WIN32)
    install(
        FILES ${CMAKE_CURRENT_SOURCE_DIR}/core/target/release/${RUST_IMPORT_LIB_NAME}
        DESTINATION ${CMAKE_INSTALL_LIBDIR}
    )
endif()
