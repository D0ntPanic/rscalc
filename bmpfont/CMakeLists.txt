cmake_minimum_required(VERSION 3.9 FATAL_ERROR)

project(bmpfont)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
find_package(Qt5 COMPONENTS Core Gui Widgets REQUIRED)

file(GLOB SOURCES *.cpp *.h)
list(FILTER SOURCES EXCLUDE REGEX moc_.*)
list(FILTER SOURCES EXCLUDE REGEX qrc_.*)

add_executable(bmpfont ${SOURCES})

set_target_properties(bmpfont PROPERTIES
	CXX_STANDARD 17
	CXX_STANDARD_REQUIRED ON
	CXX_VISIBILITY_PRESET hidden
	VISIBILITY_INLINES_HIDDEN ON
	POSITION_INDEPENDENT_CODE ON)

if(WIN32)
	set_target_properties(binaryninja PROPERTIES
		LINK_FLAGS /SUBSYSTEM:WINDOWS)
	target_link_libraries(binaryninja Qt5::WinMain)
endif()

target_link_libraries(bmpfont Qt5::Core Qt5::Gui Qt5::Widgets)

