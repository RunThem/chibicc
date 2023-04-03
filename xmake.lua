set_project('chibicc')

set_xmakever('2.6.1')

set_version('0.0.1')

add_rules('mode.debug', 'mode.release')

-- set_toolchains('@clang')
add_cflags('-fblocks', '-std=gnu2x')

-- set warning all as error
set_warnings('all')

-- add defines
add_defines('_GNU_SOURCE')

if is_plat('linux') then
  add_ldflags('-lBlocksRuntime')
end

if is_mode('debug') then
  add_cflags('-pg')
  add_undefines('NDEBUG')
end

if is_mode('release') then
  add_defines('NDEBUG')
end

add_repositories('RunThem https://github.com/RunThem/My-xmake-repo')
add_requires('mimalloc')

target('chibicc', function()
  set_kind('binary')
  add_files('*.c')
  set_targetdir('./')
  add_packages('mimalloc')
end)

target('test', function()
  set_kind('phony')
  add_deps('chibicc')

  on_run(function(T)
    os.exec('./test.sh')
  end)
end)
