# 前后端认证接口对接文档

本文档详细说明了若依Rust后端的认证相关接口和前端对接方式。

## 认证流程

1. 前端请求验证码
2. 用户填写用户名、密码和验证码
3. 前端发送登录请求
4. 后端验证用户名、密码和验证码
5. 验证通过后，后端返回JWT令牌
6. 前端存储令牌，并在后续请求中携带
7. 前端请求获取用户信息和菜单

## 接口说明

### 1. 获取验证码

**请求方式**：GET

**URL**：`/captchaImage`

**参数**：无

**响应示例**：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "img": "Base64编码的图片",
    "uuid": "验证码唯一标识",
    "captchaEnabled": true
  }
}
```

**使用方法**：

1. 前端获取到验证码图片和uuid
2. 将图片显示在登录界面
3. 将uuid保存，用于登录请求

### 2. 用户登录

**请求方式**：POST

**URL**：`/login`

**请求体**：

```json
{
  "username": "admin",
  "password": "admin123",
  "code": "1234",
  "uuid": "验证码唯一标识",
  "rememberMe": false
}
```

**响应示例**：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

**使用方法**：

1. 前端将用户输入的用户名、密码、验证码和之前保存的uuid一起发送给后端
2. 获取到token后，将其存储在本地存储中（如localStorage或Cookie）
3. 在后续的请求中，在请求头中添加`Authorization: Bearer {token}`

### 3. 获取用户信息

**请求方式**：GET

**URL**：`/getInfo`

**请求头**：`Authorization: Bearer {token}`

**响应示例**：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "userId": 1,
    "userName": "admin",
    "nickName": "若依管理员",
    "avatar": "/profile/avatar/2023/11/20/avatar.jpg",
    "permissions": ["*:*:*"],
    "roles": ["admin"]
  }
}
```

**使用方法**：

1. 前端在登录成功后调用此接口
2. 获取用户的基本信息、权限和角色
3. 根据返回的权限和角色，动态构建前端路由和权限控制

### 4. 获取用户菜单

**请求方式**：GET

**URL**：`/api/system/menu/user-menu-tree`

**请求头**：`Authorization: Bearer {token}`

**响应示例**：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "id": 1,
      "parentId": 0,
      "name": "系统管理",
      "icon": "system",
      "path": "/system",
      "component": "Layout",
      "type": "M",
      "orderNum": 1,
      "visible": "0",
      "status": "0",
      "children": [
        {
          "id": 100,
          "parentId": 1,
          "name": "用户管理",
          "icon": "user",
          "path": "user",
          "component": "system/user/index",
          "type": "C",
          "orderNum": 1,
          "visible": "0",
          "status": "0",
          "perms": "system:user:list"
        }
      ]
    }
  ]
}
```

**使用方法**：

1. 前端在获取用户信息后调用此接口
2. 根据返回的菜单结构，动态生成前端的导航菜单

### 5. 退出登录

**请求方式**：POST

**URL**：`/logout`

**请求头**：`Authorization: Bearer {token}`

**响应示例**：

```json
{
  "code": 200,
  "msg": "退出成功",
  "data": null
}
```

**使用方法**：

1. 前端在用户点击退出时调用此接口
2. 清除本地存储的token和用户信息
3. 跳转到登录页面

## 安全建议

1. 所有涉及到敏感信息的请求都应该使用HTTPS
2. 前端应该对用户输入进行基本的验证
3. 后端应该对所有请求进行验证码和权限验证
4. JWT令牌应该设置合理的过期时间
5. 敏感操作应该要求用户重新验证
6. 密码应该在前端进行加密处理

## 错误码说明

- 200: 成功
- 401: 未授权（如令牌无效或过期）
- 403: 禁止访问（权限不足）
- 500: 服务器内部错误

## 前端示例代码

```javascript
// 登录示例
async function login(username, password, code, uuid) {
  try {
    const response = await axios.post('/login', {
      username,
      password,
      code,
      uuid,
      rememberMe: false
    });
    
    if (response.data.code === 200) {
      const token = response.data.data.token;
      localStorage.setItem('token', token);
      axios.defaults.headers.common['Authorization'] = `Bearer ${token}`;
      return true;
    }
    return false;
  } catch (error) {
    console.error('登录失败', error);
    return false;
  }
}

// 获取用户信息示例
async function getUserInfo() {
  try {
    const response = await axios.get('/getInfo');
    if (response.data.code === 200) {
      return response.data.data;
    }
    return null;
  } catch (error) {
    console.error('获取用户信息失败', error);
    return null;
  }
}
``` 