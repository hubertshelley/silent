# 路由支持

### 路由匹配

路由匹配是通过每一级的路由名称来匹配的，例如：`/user/list`，会依次匹配`/`、`/user`、`/user/list`，如果匹配到了，就会执行对应的路由处理函数。

代码示例:

```rust
// 引入Silent
use silent::prelude::*;

// 定义处理方法
async fn hello_world<'a>(_req: Request) -> Result<&'a str, SilentError> {
    Ok("Hello World")
}

fn main() {
    // 定义路由
    let route = Route::new("hello_world").get(hello_world);

    // 运行服务
    Server::new()
        .bind_route(route)
        .run();
}
```

### 路由参数

路由参数是通过`<key:type>`来定义的，例如：`/user/<id>`，会匹配`/user/1`、`/user/2`等等，同时会将`id`作为参数传递给路由处理函数。

1. 路由参数类型:

   | 类型定义      | 匹配类型    | 示例                |
       |-----------|---------|-------------------|
   | str       | 字符串     | `<key:str>`       |
   | int       | 整形      | `<key:int>`       |
   | uuid      | UUID    | `<key:uuid>`      |
   | path      | 当前URL   | `<key:path>`      |
   | full_path | 后续所有URL | `<key:full_path>` |
   | *         | 当前URL   | `<key:*>`         |
   | **        | 后续所有URL | `<key:**>`        |
   | 缺省        | 字符串     | `<key>`           |
   | 其他        | 字符串     | `<key:other..>`   |

2. 代码示例:

 ```rust
fn main() {
   // 定义路由
   let route = Route::new("path_params")
           .append(Route::new("<key:str>").get(hello_world))
           .append(Route::new("<key:int>").get(hello_world))
           .append(Route::new("<key:uuid>").get(hello_world))
           .append(Route::new("<key:path>").get(hello_world))
           .append(Route::new("<key:full_path>").get(hello_world))
           .append(Route::new("<key:*>").get(hello_world))
           .append(Route::new("<key:**>").get(hello_world))
           .append(Route::new("<key>").get(hello_world))
           .append(Route::new("<key:other>").get(hello_world));
}

// 定义处理方法
async fn hello_world<'a>(req: Request) -> Result<&'a str, SilentError> {
   let path_params = req.get_path_params("key");
   Ok("Hello World")
}
 ```
