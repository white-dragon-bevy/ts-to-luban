/**
 * Luban Schema 自动生成器
 *
 * 功能说明:
 *   支持两种模式将 TypeScript 类转换为 Luban XML bean 定义：
 *   1. 注册文件模式（--registrations）：从注册文件中读取通过 registerClass() 注册的类
 *   2. 目录扫描模式（--input）：递归扫描目录，自动转换所有导出的类
 *
 * 核心特性:
 *   • 增量编译：只重新生成变化的 bean（基于文件 hash）
 *   • 类型映射：自动将 TS 类型转换为 Luban 类型
 *   • 注释提取：支持类注释和 @param 参数注释
 *   • 路径解析：支持本地模块、npm 包、路径别名
 *   • 目录扫描：递归扫描目录，自动发现所有导出的类
 *   • 智能过滤：自动排除声明文件和测试文件
 *   • 两种注册方式（仅 --registrations 模式）：
 *     - reg(ClassName)                # 名称自动推断
 *     - reg("BeanName", ClassName)    # 显式指定名称
 *
 * 使用方法:
 *   node scripts/gen-luban-schema.mjs [options]
 *
 * 选项:
 *   -h, --help                 显示帮助信息
 *   --force                    强制重新生成所有 bean（忽略缓存）
 *   --registrations <path>     指定注册文件路径（相对或绝对路径）
 *                              默认: src/types/reflect/registrations.ts
 *   --input <dir>              指定目录，递归扫描并转换所有导出的类
 *                              （与 --registrations 互斥，优先使用 --input）
 *   --output <path>            指定输出 XML 文件路径（相对或绝对路径）
 *                              默认: configs/defines/reflect/generated.xml
 *
 * 示例:
 *   # 注册文件模式 - 增量生成
 *   node scripts/gen-luban-schema.mjs
 *
 *   # 强制重新生成所有 bean
 *   node scripts/gen-luban-schema.mjs --force
 *
 *   # 指定注册文件路径
 *   node scripts/gen-luban-schema.mjs --registrations src/types/reflect/registers/a.ts --output configs/defines/reflect/a.xml
 *
 *   # 目录扫描模式 - 递归扫描并转换所有导出的类
 *   node scripts/gen-luban-schema.mjs --input src/shared/bevy/visual/trigger --output configs/defines/reflect/triggers.xml
 *
 *   # 显示帮助信息
 *   node scripts/gen-luban-schema.mjs -h
 */

import fs from "fs";
import path from "path";
import crypto from "crypto";
import { fileURLToPath } from "url";
import ts from "typescript";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PROJECT_ROOT = path.join(__dirname, "..", "..", "..");  // src/types/reflect/ -> src/types/ -> src/ -> 项目根目录

// ========== 配置 ==========

// 默认路径（可通过命令行参数覆盖）
const DEFAULT_REGISTRATIONS_FILE = path.join(PROJECT_ROOT, "src/types/reflect/registrations.ts");
const DEFAULT_OUTPUT_FILE = path.join(PROJECT_ROOT, "configs/defines/reflect/generated.xml");
const TSCONFIG_PATH = path.join(PROJECT_ROOT, "tsconfig.json");

// TypeScript 类型到 Luban 类型的映射
const TYPE_MAPPING = {
    // 基础类型
    number: "int",
    string: "string",
    boolean: "bool",
    // 特殊处理
    float: "float",
    double: "double",
    int: "int",
    long: "long",
    // Roblox 类型（对应 configs/defines/roblox.xml）
    vector3: "Vector3",  // Roblox Vector3
    vector2: "Vector2",  // Roblox Vector2
    cframe: "CFrame",    // Roblox CFrame
    color3: "Color3",    // Roblox Color3
    // Bevy ECS 类型
    anyentity: "long",   // 实体 ID（运行时生成的数字）
    entity: "long",      // 实体 ID
    entityid: "long",    // 实体 ID
    assetpath: "string", // 资源路径类型
    // 施法系统类型（对应 configs/defines/bevy/core.xml）
    castactiontarget: "CastActionTarget",  // 施法目标
    castcontext: "CastContext",            // 施法上下文
};

// ========== 工具函数 ==========

/**
 * 计算文件内容的 MD5 哈希
 */
function computeFileHash(filePath) {
    if (!fs.existsSync(filePath)) return null;
    const content = fs.readFileSync(filePath, "utf-8");
    return crypto.createHash("md5").update(content).digest("hex");
}

/**
 * 解析 tsconfig.json 获取路径别名
 */
function parseTsConfig() {
    const configFile = ts.readConfigFile(TSCONFIG_PATH, ts.sys.readFile);
    const parsedConfig = ts.parseJsonConfigFileContent(configFile.config, ts.sys, PROJECT_ROOT);
    return parsedConfig;
}

/**
 * 检查是否是 npm 包路径
 */
function isNpmPackage(importPath) {
    // npm 包路径通常以 @ 开头或不以 . 或 / 开头
    return importPath.startsWith("@") || (!importPath.startsWith(".") && !importPath.startsWith("/"));
}

/**
 * 解析路径别名，将 "shared/xxx" 转换为实际路径
 * 支持本地路径别名和 npm 包
 */
function resolveModulePath(importPath, tsConfig) {
    const { paths, baseUrl } = tsConfig.options;

    // 解析文件路径，支持目录索引文件 (index.ts)
    function resolveFilePath(filePath) {
        // 如果已经有扩展名，直接返回
        if (filePath.endsWith(".ts") || filePath.endsWith(".tsx") || filePath.endsWith(".d.ts")) {
            return filePath;
        }

        // 先检查是否存在目录，如果是目录则使用 index.ts
        if (fs.existsSync(filePath) && fs.statSync(filePath).isDirectory()) {
            const indexPath = path.join(filePath, "index.ts");
            if (fs.existsSync(indexPath)) {
                return indexPath;
            }
        }

        // 检查 .ts 文件是否存在
        const tsPath = filePath + ".ts";
        if (fs.existsSync(tsPath)) {
            return tsPath;
        }

        // 检查 .tsx 文件
        const tsxPath = filePath + ".tsx";
        if (fs.existsSync(tsxPath)) {
            return tsxPath;
        }

        // 如果都不存在，默认返回 .ts 路径（可能是声明文件）
        return tsPath;
    }

    // 优先检查 npm 包（以 @ 开头的是 npm 包）
    if (importPath.startsWith("@")) {
        const npmPath = path.join(PROJECT_ROOT, "node_modules", importPath, "index.d.ts");
        return { path: npmPath, isNpmPackage: true, packageName: importPath };
    }

    // 然后检查路径别名（排除通配符 "*" 匹配，因为它会误匹配 npm 包）
    if (paths && baseUrl) {
        for (const [pattern, replacements] of Object.entries(paths)) {
            // 跳过纯通配符模式 "*"
            if (pattern === "*") continue;

            const regex = new RegExp("^" + pattern.replace("*", "(.*)") + "$");
            const match = importPath.match(regex);

            if (match) {
                const replacement = replacements[0].replace("*", match[1] || "");
                return { path: resolveFilePath(path.join(baseUrl, replacement)), isNpmPackage: false };
            }
        }

        // 最后尝试使用 "*" 通配符模式
        if (paths["*"]) {
            const replacement = paths["*"][0].replace("*", importPath);
            return { path: resolveFilePath(path.join(baseUrl, replacement)), isNpmPackage: false };
        }
    }

    // 默认为本地路径
    return { path: resolveFilePath(path.join(PROJECT_ROOT, "src", importPath)), isNpmPackage: false };
}

/**
 * 从 npm 包中查找类的定义文件
 */
function findClassInNpmPackage(packageName, className) {
    const packageRoot = path.join(PROJECT_ROOT, "node_modules", packageName);

    // 递归搜索 .d.ts 文件
    function searchDir(dir) {
        if (!fs.existsSync(dir)) return null;

        const entries = fs.readdirSync(dir, { withFileTypes: true });
        for (const entry of entries) {
            const fullPath = path.join(dir, entry.name);

            if (entry.isDirectory() && !entry.name.startsWith("__")) {
                const result = searchDir(fullPath);
                if (result) return result;
            } else if (entry.isFile() && entry.name.endsWith(".d.ts")) {
                // 读取文件内容检查是否包含类定义
                const content = fs.readFileSync(fullPath, "utf-8");
                if (content.includes(`class ${className}`) || content.includes(`export declare class ${className}`)) {
                    return fullPath;
                }
            }
        }
        return null;
    }

    return searchDir(packageRoot);
}

/**
 * 从本地目录中查找类的定义文件
 */
function findClassInDirectory(dir, className) {
    if (!fs.existsSync(dir)) return null;

    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);

        // 只搜索 .ts 和 .tsx 文件，跳过 index.ts
        if (entry.isFile() &&
            (entry.name.endsWith(".ts") || entry.name.endsWith(".tsx")) &&
            entry.name !== "index.ts" &&
            entry.name !== "index.tsx") {

            // 读取文件内容检查是否包含类定义
            const content = fs.readFileSync(fullPath, "utf-8");
            if (content.includes(`class ${className}`) ||
                content.includes(`export class ${className}`) ||
                content.includes(`export declare class ${className}`)) {
                return fullPath;
            }
        }
    }
    return null;
}

/**
 * 从 registrations.ts 提取所有注册的类信息
 *
 * 支持两种调用模式:
 * 1. reg(ClassName) - 只传构造函数（使用宏自动获取名称）
 * 2. reg("ClassName", ClassName) - 指定名称+构造函数
 */
function extractRegistrations(sourceFile, importAliases) {
    const registrations = [];

    function visit(node) {
        if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
            const funcName = node.expression.text;

            // 检查是否是 registerClass、reg 或其别名
            const isRegisterFunc = funcName === "registerClass" || funcName === "reg" || importAliases.has(funcName);

            if (isRegisterFunc) {
                // 模式1: reg(ClassName) - 只传构造函数
                if (node.arguments.length >= 1) {
                    const firstArg = node.arguments[0];

                    // 第一个参数是标识符（类构造函数）
                    if (ts.isIdentifier(firstArg)) {
                        const className = firstArg.text;
                        registrations.push({ className });
                        return;
                    }

                    // 模式2: reg("ClassName", ClassName) - 字符串+类模式
                    if (ts.isStringLiteral(firstArg) && node.arguments.length >= 2) {
                        const secondArg = node.arguments[1];
                        if (ts.isIdentifier(secondArg)) {
                            const className = secondArg.text; // 使用实际的类标识符
                            registrations.push({ className, registeredName: firstArg.text });
                        }
                    }
                }
            }
        }

        ts.forEachChild(node, visit);
    }

    visit(sourceFile);
    return registrations;
}

/**
 * 从 import 语句中提取函数别名映射
 * 例如: import { registerClass as reg } -> Map { "reg" -> "registerClass" }
 */
function extractImportAliases(sourceFile) {
    const aliases = new Map();

    function visit(node) {
        if (ts.isImportDeclaration(node) && node.importClause) {
            if (node.importClause.namedBindings && ts.isNamedImports(node.importClause.namedBindings)) {
                for (const element of node.importClause.namedBindings.elements) {
                    // 检查是否有别名 (import { A as B })
                    if (element.propertyName) {
                        const originalName = element.propertyName.text;
                        const aliasName = element.name.text;
                        if (originalName === "registerClass") {
                            aliases.set(aliasName, originalName);
                        }
                    }
                }
            }
        }

        ts.forEachChild(node, visit);
    }

    visit(sourceFile);
    return aliases;
}

/**
 * 从 import 语句中提取类的源文件路径
 */
function extractImportPaths(sourceFile) {
    const imports = new Map();

    function visit(node) {
        if (ts.isImportDeclaration(node) && node.importClause) {
            const moduleSpecifier = node.moduleSpecifier.text;

            // 处理命名导入 import { A, B } from "xxx"
            if (node.importClause.namedBindings && ts.isNamedImports(node.importClause.namedBindings)) {
                for (const element of node.importClause.namedBindings.elements) {
                    const name = element.name.text;
                    imports.set(name, moduleSpecifier);
                }
            }

            // 处理默认导入 import A from "xxx"
            if (node.importClause.name) {
                imports.set(node.importClause.name.text, moduleSpecifier);
            }
        }

        ts.forEachChild(node, visit);
    }

    visit(sourceFile);
    return imports;
}

/**
 * 将 TypeScript 类型转换为 Luban 类型
 * @param typeNode - TypeScript 类型节点
 * @param checker - TypeScript 类型检查器
 * @param context - 上下文信息（用于错误报告）
 * @returns {{ type: string, error: string | null }} - 返回类型和错误信息
 */
function convertType(typeNode, checker, context = {}) {
    if (!typeNode) return { type: "string", error: null };

    // 处理数组类型 T[] -> list,T
    if (ts.isArrayTypeNode(typeNode)) {
        const elementResult = convertType(typeNode.elementType, checker, context);
        return {
            type: `list,${elementResult.type}`,
            error: elementResult.error
        };
    }

    // 处理联合类型 (number | UnitFlag) -> 取第一个
    if (ts.isUnionTypeNode(typeNode)) {
        return convertType(typeNode.types[0], checker, context);
    }

    // 处理类型引用
    if (ts.isTypeReferenceNode(typeNode)) {
        const typeName = typeNode.typeName.getText();

        // 特殊处理 ReadonlyArray<T> -> list,T
        if (typeName === "ReadonlyArray" && typeNode.typeArguments && typeNode.typeArguments.length > 0) {
            const elementResult = convertType(typeNode.typeArguments[0], checker, context);
            return {
                type: `list,${elementResult.type}`,
                error: elementResult.error
            };
        }

        // 特殊处理 Record<K, V> -> map,K,V
        if (typeName === "Record" && typeNode.typeArguments && typeNode.typeArguments.length === 2) {
            const keyResult = convertType(typeNode.typeArguments[0], checker, context);
            const valueResult = convertType(typeNode.typeArguments[1], checker, context);
            return {
                type: `map,${keyResult.type},${valueResult.type}`,
                error: keyResult.error || valueResult.error
            };
        }

        // 特殊处理 Map<K, V> -> map,K,V
        if (typeName === "Map" && typeNode.typeArguments && typeNode.typeArguments.length === 2) {
            const keyResult = convertType(typeNode.typeArguments[0], checker, context);
            const valueResult = convertType(typeNode.typeArguments[1], checker, context);
            return {
                type: `map,${keyResult.type},${valueResult.type}`,
                error: keyResult.error || valueResult.error
            };
        }

        // 特殊处理 Array<[K, V]> -> map,K,V (元组数组)
        if (typeName === "Array" && typeNode.typeArguments && typeNode.typeArguments.length === 1) {
            const elementType = typeNode.typeArguments[0];
            // 检查元素是否是元组类型
            if (ts.isTupleTypeNode(elementType) && elementType.elements.length === 2) {
                const keyResult = convertType(elementType.elements[0], checker, context);
                const valueResult = convertType(elementType.elements[1], checker, context);
                return {
                    type: `map,${keyResult.type},${valueResult.type}`,
                    error: keyResult.error || valueResult.error
                };
            }
        }

        // 检查是否是已知类型映射
        if (TYPE_MAPPING[typeName.toLowerCase()]) {
            return { type: TYPE_MAPPING[typeName.toLowerCase()], error: null };
        }

        // 尝试获取类型别名的实际类型
        const type = checker.getTypeAtLocation(typeNode);
        if (type.isStringLiteral()) {
            return { type: "string", error: null };
        }
        if (type.isNumberLiteral()) {
            return { type: "int", error: null };
        }

        // 检查是否是字符串字面量联合类型（如 "grant" | "revoke"）
        if (type.isUnion()) {
            const allStrings = type.types.every((t) => t.isStringLiteral());
            if (allStrings) {
                return { type: "string", error: null };
            }
        }

        // 检查是否是接口类型（Interface）
        if (type.symbol) {
            const declarations = type.symbol.declarations;
            if (declarations && declarations.length > 0) {
                const isInterface = declarations.some((decl) => ts.isInterfaceDeclaration(decl));
                if (isInterface) {
                    // 检查是否是 EntityTrigger 接口或继承自 EntityTrigger 的接口
                    if (typeName === "EntityTrigger" || hasInterfaceInChain(type, "EntityTrigger")) {
                        // EntityTrigger 接口映射为 TsTriggerClass
                        return {
                            type: "TsTriggerClass",
                            error: null
                        };
                    }

                    // 检查接口是否被导出（有 export 修饰符）
                    // 如果是导出的接口，直接使用接口名称作为类型（会被生成为独立 bean）
                    const interfaceDecl = declarations.find((decl) => ts.isInterfaceDeclaration(decl));
                    if (interfaceDecl) {
                        const hasExportModifier = interfaceDecl.modifiers?.some(
                            (m) => m.kind === ts.SyntaxKind.ExportKeyword
                        );
                        if (hasExportModifier) {
                            // 导出的接口类型直接使用接口名称
                            return {
                                type: typeName,
                                error: null
                            };
                        }
                    }

                    // 非导出的接口类型映射为 TsClass（Luban 多态基类）
                    return {
                        type: "TsClass",
                        error: null
                    };
                }
            }
        }

        // 未知类型：返回错误信息
        const errorMsg = `未知类型 '${typeName}' (${context.className}.${context.fieldName})`;
        return {
            type: typeName.toLowerCase(),
            error: errorMsg
        };
    }

    // 处理基础类型关键字
    if (ts.isLiteralTypeNode(typeNode)) {
        if (typeNode.literal.kind === ts.SyntaxKind.StringLiteral) {
            return { type: "string", error: null };
        }
        if (typeNode.literal.kind === ts.SyntaxKind.NumericLiteral) {
            return { type: "int", error: null };
        }
    }

    // 关键字类型
    switch (typeNode.kind) {
        case ts.SyntaxKind.NumberKeyword:
            return { type: "int", error: null };
        case ts.SyntaxKind.StringKeyword:
            return { type: "string", error: null };
        case ts.SyntaxKind.BooleanKeyword:
            return { type: "bool", error: null };
        default:
            return { type: "string", error: null };
    }
}

/**
 * 检查类是否实现了指定接口（递归检查接口继承链）
 * @param classNode - 类声明节点
 * @param interfaceName - 要检查的接口名称
 * @param checker - TypeScript 类型检查器
 * @returns 是否实现了该接口
 */
function classImplementsInterface(classNode, interfaceName, checker) {
    if (!classNode.name) return false;

    const className = classNode.name.text;

    // 方法1: 直接解析 implements 子句（适用于 .d.ts 文件）
    if (classNode.heritageClauses) {
        for (const clause of classNode.heritageClauses) {
            // implements 子句的 token 是 ImplementsKeyword
            if (clause.token === ts.SyntaxKind.ImplementsKeyword) {
                for (const type of clause.types) {
                    // 获取接口名称（忽略泛型参数）
                    const typeName = type.expression.getText();

                    if (typeName === interfaceName) {
                        return true;
                    }

                    // 递归检查接口继承
                    const implementedType = checker.getTypeAtLocation(type.expression);
                    if (hasInterfaceInChain(implementedType, interfaceName)) {
                        return true;
                    }
                }
            }
        }
    }

    // 方法2: 使用类型检查器获取基类型（备用方法）
    const classType = checker.getTypeAtLocation(classNode);
    const implementedInterfaces = classType.getBaseTypes() || [];

    for (const baseType of implementedInterfaces) {
        if (baseType.symbol?.name === interfaceName) {
            return true;
        }

        if (hasInterfaceInChain(baseType, interfaceName)) {
            return true;
        }
    }

    return false;
}

/**
 * 递归检查接口继承链中是否包含指定接口
 * @param type - TypeScript 类型
 * @param targetName - 目标接口名称
 * @returns 是否在继承链中
 */
function hasInterfaceInChain(type, targetName) {
    if (type.symbol?.name === targetName) {
        return true;
    }

    const baseTypes = type.getBaseTypes() || [];
    for (const baseType of baseTypes) {
        if (hasInterfaceInChain(baseType, targetName)) {
            return true;
        }
    }

    return false;
}

/**
 * 从类定义中提取字段信息
 * @returns {classInfo, errors} - 返回类信息和错误列表
 */
function extractClassFields(classFilePath, className, program) {
    const sourceFile = program.getSourceFile(classFilePath);
    if (!sourceFile) {
        console.warn(`  警告: 无法读取源文件 ${classFilePath}`);
        return { classInfo: null, errors: [] };
    }

    // 计算源文件的 hash
    const fileHash = computeFileHash(classFilePath);

    const checker = program.getTypeChecker();
    let classInfo = null;
    const typeErrors = [];  // 收集类型错误

    function visit(node) {
        if (ts.isClassDeclaration(node) && node.name && node.name.text === className) {
            const fields = [];
            let classComment = "";

            // 检查类是否实现了 EntityTrigger 接口
            const isEntityTrigger = classImplementsInterface(node, "EntityTrigger", checker);

            // 获取类的 JSDoc 注释
            const leadingComments = ts.getLeadingCommentRanges(sourceFile.getFullText(), node.getFullStart());
            if (leadingComments && leadingComments.length > 0) {
                const lastComment = leadingComments[leadingComments.length - 1];
                const commentText = sourceFile.getFullText().slice(lastComment.pos, lastComment.end);

                // 提取 JSDoc 注释的描述部分（忽略 @tags）
                // 匹配 /** 和第一个 @ 或 */ 之间的内容
                const descMatch = commentText.match(/\/\*\*\s*([\s\S]*?)(?:@|\*\/)/);
                if (descMatch) {
                    // 清理每行的 * 前缀和空白
                    const rawDesc = descMatch[1];
                    const lines = rawDesc.split('\n')
                        .map(line => line.replace(/^\s*\*\s?/, '').trim())
                        .filter(line => line.length > 0);

                    // 取第一行非空行作为简短描述
                    if (lines.length > 0) {
                        classComment = lines[0];
                    }
                }
            }

            // 判断是否是 .d.ts 文件（声明文件）
            const isDtsFile = classFilePath.endsWith(".d.ts");

            // 遍历类成员
            for (const member of node.members) {
                // 处理 constructor 参数属性
                if (ts.isConstructorDeclaration(member)) {
                    // 获取构造函数的 JSDoc 注释，提取所有 @param 标签
                    const constructorJsDocs = ts.getJSDocTags(member);
                    const paramComments = new Map();

                    for (const tag of constructorJsDocs) {
                        if (tag.tagName.text === "param" && tag.comment) {
                            const paramName = tag.name?.getText() || "";
                            const commentText = typeof tag.comment === "string"
                                ? tag.comment
                                : tag.comment.map(c => c.text).join("");
                            // 移除开头的 " - " 分隔符
                            const cleanComment = commentText.trim().replace(/^-\s*/, "");
                            paramComments.set(paramName, cleanComment);
                        }
                    }

                    for (const param of member.parameters) {
                        // 对于 .d.ts 文件，所有构造函数参数都被视为公开字段
                        // 对于 .ts 文件，检查是否有任何参数修饰符（public/private/protected/readonly）
                        // 有修饰符的参数会被提升为类属性，应该被提取
                        const hasParameterPropertyModifier = param.modifiers?.some(
                            (m) => m.kind === ts.SyntaxKind.PublicKeyword ||
                                   m.kind === ts.SyntaxKind.PrivateKeyword ||
                                   m.kind === ts.SyntaxKind.ProtectedKeyword ||
                                   m.kind === ts.SyntaxKind.ReadonlyKeyword,
                        );

                        const shouldInclude = isDtsFile || hasParameterPropertyModifier;

                        if (shouldInclude && ts.isIdentifier(param.name)) {
                            const fieldName = param.name.text;

                            // 跳过 nominal typing 标记字段
                            if (fieldName.includes("_nominal_")) {
                                continue;
                            }

                            // 跳过 Trigger Combinator 的内部标记字段
                            if (fieldName === "_is_trigger_combinator" || fieldName === "_trigger_type") {
                                continue;
                            }

                            const typeResult = convertType(param.type, checker, { className, fieldName });

                            // 收集类型错误
                            if (typeResult.error) {
                                typeErrors.push(typeResult.error);
                            }

                            // 从 @param 标签获取注释
                            const comment = paramComments.get(fieldName) || "";

                            fields.push({
                                name: fieldName,
                                type: typeResult.type,
                                comment: comment,
                                isOptional: !!param.questionToken,
                            });
                        }
                    }
                }

                // 处理普通属性声明
                if (ts.isPropertyDeclaration(member) && ts.isIdentifier(member.name)) {
                    const hasPublicModifier =
                        !member.modifiers ||
                        member.modifiers.every(
                            (m) =>
                                m.kind !== ts.SyntaxKind.PrivateKeyword &&
                                m.kind !== ts.SyntaxKind.ProtectedKeyword,
                        );

                    if (hasPublicModifier) {
                        const fieldName = member.name.text;

                        // 跳过 nominal typing 标记字段
                        if (fieldName.includes("_nominal_")) {
                            continue;
                        }

                        // 跳过 Trigger Combinator 的内部标记字段
                        if (fieldName === "_is_trigger_combinator" || fieldName === "_trigger_type") {
                            continue;
                        }

                        const typeResult = convertType(member.type, checker, { className, fieldName });

                        // 收集类型错误
                        if (typeResult.error) {
                            typeErrors.push(typeResult.error);
                        }

                        // 获取注释
                        let comment = "";
                        const propComments = ts.getLeadingCommentRanges(
                            sourceFile.getFullText(),
                            member.getFullStart(),
                        );
                        if (propComments && propComments.length > 0) {
                            const commentText = sourceFile
                                .getFullText()
                                .slice(propComments[0].pos, propComments[0].end);
                            const match = commentText.match(/\*\s*([^\n\r*]+)|\/{2}\s*(.+)/);
                            if (match) {
                                comment = (match[1] || match[2] || "").trim();
                            }
                        }

                        fields.push({
                            name: fieldName,
                            type: typeResult.type,
                            comment: comment,
                            isOptional: !!member.questionToken,
                        });
                    }
                }
            }

            classInfo = {
                name: className,
                comment: classComment,
                fields: fields,
                hash: fileHash,
                isEntityTrigger: isEntityTrigger,  // 标记是否实现了 EntityTrigger 接口
            };
        }

        ts.forEachChild(node, visit);
    }

    visit(sourceFile);
    return { classInfo, errors: typeErrors };
}

/**
 * 从接口定义中提取字段信息
 * @returns {interfaceInfo, errors} - 返回接口信息和错误列表
 */
function extractInterfaceFields(interfaceFilePath, interfaceName, program) {
    const sourceFile = program.getSourceFile(interfaceFilePath);
    if (!sourceFile) {
        console.warn(`  警告: 无法读取源文件 ${interfaceFilePath}`);
        return { interfaceInfo: null, errors: [] };
    }

    // 计算源文件的 hash
    const fileHash = computeFileHash(interfaceFilePath);

    const checker = program.getTypeChecker();
    let interfaceInfo = null;
    const typeErrors = [];  // 收集类型错误

    function visit(node) {
        if (ts.isInterfaceDeclaration(node) && node.name && node.name.text === interfaceName) {
            const fields = [];
            let interfaceComment = "";

            // 获取接口的 JSDoc 注释
            const leadingComments = ts.getLeadingCommentRanges(sourceFile.getFullText(), node.getFullStart());
            if (leadingComments && leadingComments.length > 0) {
                const lastComment = leadingComments[leadingComments.length - 1];
                const commentText = sourceFile.getFullText().slice(lastComment.pos, lastComment.end);

                // 提取 JSDoc 注释的描述部分（忽略 @tags）
                const descMatch = commentText.match(/\/\*\*\s*([\s\S]*?)(?:@|\*\/)/);
                if (descMatch) {
                    const rawDesc = descMatch[1];
                    const lines = rawDesc.split('\n')
                        .map(line => line.replace(/^\s*\*\s?/, '').trim())
                        .filter(line => line.length > 0);

                    if (lines.length > 0) {
                        interfaceComment = lines[0];
                    }
                }
            }

            // 遍历接口成员
            for (const member of node.members) {
                // 处理属性签名
                if (ts.isPropertySignature(member) && ts.isIdentifier(member.name)) {
                    const fieldName = member.name.text;

                    const typeResult = convertType(member.type, checker, { className: interfaceName, fieldName });

                    // 收集类型错误
                    if (typeResult.error) {
                        typeErrors.push(typeResult.error);
                    }

                    // 获取属性注释
                    let comment = "";
                    const propComments = ts.getLeadingCommentRanges(
                        sourceFile.getFullText(),
                        member.getFullStart(),
                    );
                    if (propComments && propComments.length > 0) {
                        const commentText = sourceFile
                            .getFullText()
                            .slice(propComments[0].pos, propComments[0].end);
                        const match = commentText.match(/\*\s*([^\n\r*]+)|\/{2}\s*(.+)/);
                        if (match) {
                            comment = (match[1] || match[2] || "").trim();
                        }
                    }

                    fields.push({
                        name: fieldName,
                        type: typeResult.type,
                        comment: comment,
                        isOptional: !!member.questionToken,
                    });
                }
            }

            interfaceInfo = {
                name: interfaceName,
                comment: interfaceComment,
                fields: fields,
                hash: fileHash,
                isInterface: true,  // 标记为接口
            };
        }

        ts.forEachChild(node, visit);
    }

    visit(sourceFile);
    return { interfaceInfo, errors: typeErrors };
}

/**
 * 解析现有 XML 文件中的 bean 定义和 hash
 */
function parseExistingXml(xmlPath) {
    if (!fs.existsSync(xmlPath)) {
        return new Map(); // 返回空 Map，className -> { hash, xml }
    }

    const content = fs.readFileSync(xmlPath, "utf-8");
    const beanMap = new Map();

    // 正则匹配 bean 定义（包括前面的 hash 注释）
    const beanPattern = /(?:<!--\s*hash:(\w+)\s*-->\n\s*)?<bean name="([^"]+)"[^>]*>[\s\S]*?<\/bean>/g;
    let match;

    while ((match = beanPattern.exec(content)) !== null) {
        const hash = match[1] || null;
        const className = match[2];
        const beanXml = match[0];

        beanMap.set(className, { hash, xml: beanXml });
    }

    return beanMap;
}

/**
 * 生成 XML bean 定义
 * @param classes - 类信息数组
 */
function generateXml(classes) {
    const lines = [
        '<?xml version="1.0" encoding="utf-8"?>',
        "<!--",
        "  Luban Schema 自动生成文件",
        "  由 scripts/gen-luban-schema.mjs 生成",
        "  请勿手动修改此文件",
        "-->",
        '<module name="" comment="自动生成的 ts class Bean 定义">',
        "",
    ];

    for (const cls of classes) {
        // 如果有缓存的 XML，直接使用
        if (cls.cachedXml) {
            // 直接添加缓存的 XML（已经包含正确的缩进）
            lines.push(cls.cachedXml);
            lines.push("");
            continue;
        }

        // 否则生成新的 XML
        const commentAttr = cls.comment ? ` comment="${escapeXml(cls.comment)}"` : "";

        // 为每个 bean 添加 hash 注释
        if (cls.hash) {
            lines.push(`    <!-- hash:${cls.hash} -->`);
        }

        // 根据类型选择是否需要父类
        // - 接口类型 -> 不需要父类，独立 bean
        // - 实现了 EntityTrigger -> 继承 TsTriggerClass
        // - 其他类 -> 继承 TsClass
        if (cls.isInterface) {
            lines.push(`    <bean name="${cls.name}"${commentAttr}>`);
        } else {
            const parentClass = cls.isEntityTrigger ? "TsTriggerClass" : "TsClass";
            lines.push(`    <bean name="${cls.name}" parent="${parentClass}"${commentAttr}>`);
        }

        // 生成字段定义
        // 注意：即使没有字段也不添加占位字段，因为 TsClass 父类已经有 _placeholder 字段
        for (const field of cls.fields) {
            const fieldCommentAttr = field.comment ? ` comment="${escapeXml(field.comment)}"` : "";

            // 处理可选类型
            let fieldType = field.type;
            if (field.isOptional) {
                // 对于 list 类型，忽略可选标记（空数组和 undefined 等价）
                if (!fieldType.startsWith("list,")) {
                    fieldType = `${fieldType}?`;
                }
            }

            lines.push(`        <var name="${field.name}" type="${fieldType}"${fieldCommentAttr}/>`);
        }

        lines.push("    </bean>");
        lines.push("");
    }

    lines.push("</module>");

    return lines.join("\n");
}

/**
 * 递归扫描目录，查找所有 .ts 文件（排除 .d.ts、.spec.ts、.test.ts）
 */
function scanDirectoryForTsFiles(dirPath) {
    const tsFiles = [];

    function scan(dir) {
        if (!fs.existsSync(dir)) {
            console.warn(`  警告: 目录不存在 ${dir}`);
            return;
        }

        const entries = fs.readdirSync(dir, { withFileTypes: true });

        for (const entry of entries) {
            const fullPath = path.join(dir, entry.name);

            if (entry.isDirectory()) {
                // 递归扫描子目录
                scan(fullPath);
            } else if (entry.isFile()) {
                // 只处理 .ts 和 .tsx 文件，排除声明文件和测试文件
                if ((entry.name.endsWith(".ts") || entry.name.endsWith(".tsx")) &&
                    !entry.name.endsWith(".d.ts") &&
                    !entry.name.endsWith(".spec.ts") &&
                    !entry.name.endsWith(".test.ts")) {
                    tsFiles.push(fullPath);
                }
            }
        }
    }

    scan(dirPath);
    return tsFiles;
}

/**
 * 从源文件中提取所有导出的类和接口
 * 返回格式: [{ className: string, filePath: string, isInterface: boolean }]
 */
function extractExportedClasses(sourceFile, filePath) {
    const exportedItems = [];

    function visit(node) {
        // 查找导出的类声明
        if (ts.isClassDeclaration(node) && node.name) {
            const className = node.name.text;

            // 检查是否有 export 修饰符
            const hasExportModifier = node.modifiers?.some(
                (m) => m.kind === ts.SyntaxKind.ExportKeyword
            );

            if (hasExportModifier) {
                exportedItems.push({
                    className,
                    filePath,
                    isInterface: false,
                });
            }
        }

        // 查找导出的接口声明
        if (ts.isInterfaceDeclaration(node) && node.name) {
            const interfaceName = node.name.text;

            // 检查是否有 export 修饰符
            const hasExportModifier = node.modifiers?.some(
                (m) => m.kind === ts.SyntaxKind.ExportKeyword
            );

            if (hasExportModifier) {
                exportedItems.push({
                    className: interfaceName,
                    filePath,
                    isInterface: true,
                });
            }
        }

        ts.forEachChild(node, visit);
    }

    visit(sourceFile);
    return exportedItems;
}

/**
 * 转义 XML 特殊字符
 */
function escapeXml(str) {
    return str
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&apos;");
}

// ========== 主流程 ==========

/**
 * 显示帮助信息
 */
function showHelp() {
    console.log(`
Luban Schema 自动生成器
${"=".repeat(80)}

功能说明:
  支持两种模式将 TypeScript 类转换为 Luban XML bean 定义：
  • 注册文件模式（--registrations）：从注册文件中读取通过 registerClass() 注册的类
  • 目录扫描模式（--input）：递归扫描目录，自动转换所有导出的类

核心特性:
  • 增量编译：只重新生成变化的 bean（基于文件 hash）
  • 类型映射：自动将 TS 类型转换为 Luban 类型
  • 注释提取：支持类注释和 @param 参数注释
  • 路径解析：支持本地模块、npm 包、路径别名
  • 目录扫描：递归扫描目录，自动发现所有导出的类
  • 智能过滤：自动排除声明文件（.d.ts）和测试文件（.spec.ts、.test.ts）
  • 两种注册方式（仅 --registrations 模式）：
    - reg(ClassName)                # 名称自动推断
    - reg("BeanName", ClassName)    # 显式指定名称

使用方法:
  node scripts/gen-luban-schema.mjs [options]

选项:
  -h, --help                 显示此帮助信息
  --force                    强制重新生成所有 bean（忽略缓存）
  --registrations <path>     指定注册文件路径（相对或绝对路径）
                             默认: src/types/reflect/registrations.ts
  --input <dir>              指定目录，自动扫描并转换所有导出的类
                             （与 --registrations 互斥，优先使用 --input）
  --output <path>            指定输出 XML 文件路径（相对或绝对路径）
                             默认: configs/defines/reflect/generated.xml

示例:
  ┌─ 注册文件模式 ─────────────────────────────────────────────────────┐
  │ # 增量生成（使用默认注册文件）                                       │
  │ node scripts/gen-luban-schema.mjs                                  │
  │                                                                    │
  │ # 强制重新生成所有 bean                                             │
  │ node scripts/gen-luban-schema.mjs --force                          │
  │                                                                    │
  │ # 指定自定义注册文件                                                │
  │ node scripts/gen-luban-schema.mjs \\                               │
  │   --registrations src/types/reflect/registers/a.ts \\             │
  │   --output configs/defines/reflect/a.xml                          │
  └────────────────────────────────────────────────────────────────────┘

  ┌─ 目录扫描模式 ─────────────────────────────────────────────────────┐
  │ # 递归扫描目录并转换所有导出的类                                     │
  │ node scripts/gen-luban-schema.mjs \\                               │
  │   --input src/shared/bevy/visual/trigger \\                       │
  │   --output configs/defines/reflect/triggers.xml                   │
  │                                                                    │
  │ # 强制重新生成（目录扫描模式）                                       │
  │ node scripts/gen-luban-schema.mjs \\                               │
  │   --input src/shared/bevy/visual/controllers \\                   │
  │   --output configs/defines/reflect/controllers.xml --force        │
  └────────────────────────────────────────────────────────────────────┘

  # 显示帮助信息
  node scripts/gen-luban-schema.mjs -h

类型映射规则:
  TypeScript 类型          Luban 类型
  ────────────────────────────────────
  number                   int
  string                   string
  boolean                  bool
  Vector3                  Vector3
  AnyEntity                long
  Array<T>                 list,T
  T | undefined            T?

支持的输入模式:
  ┌─ 模式 1：注册文件模式（--registrations）──────────────────────────┐
  │ 从注册文件中读取 reg() 调用，支持以下格式：                         │
  │                                                                   │
  │ 1. 从外部导入的类:                                                │
  │    import { MyClass } from "shared/components";                  │
  │    reg(MyClass)                                                  │
  │                                                                   │
  │ 2. 本地定义的类:                                                  │
  │    class MyClass { ... }                                         │
  │    reg(MyClass)                                                  │
  │                                                                   │
  │ 3. 显式指定 bean 名称:                                            │
  │    reg("CustomBeanName", MyClass)                                │
  └───────────────────────────────────────────────────────────────────┘

  ┌─ 模式 2：目录扫描模式（--input）──────────────────────────────────┐
  │ 递归扫描指定目录，自动提取所有导出的类（export class）              │
  │                                                                   │
  │ • 自动扫描：递归搜索所有 .ts 和 .tsx 文件                          │
  │ • 智能过滤：自动排除 .d.ts、.spec.ts、.test.ts                     │
  │ • 自动提取：只提取带 export 关键字的类                             │
  │ • Bean 名称：使用类名作为 bean 名称                                │
  │                                                                   │
  │ 示例文件结构：                                                     │
  │   src/shared/bevy/visual/trigger/                                │
  │   ├── box-explosion-trigger.ts      # export class BoxExplosionTrigger  │
  │   ├── circle-explosion-trigger.ts   # export class CircleExplosionTrigger │
  │   └── sector-explosion-trigger.ts   # export class SectorExplosionTrigger │
  │                                                                   │
  │ 运行命令：                                                         │
  │   node scripts/gen-luban-schema.mjs --input src/shared/bevy/visual/trigger │
  │                                                                   │
  │ 生成结果：自动为 3 个导出的类生成 bean 定义                        │
  └───────────────────────────────────────────────────────────────────┘

注意事项:
  • 只有 public 构造函数参数和属性会被导出
  • 支持可选类型（?:）和数组类型
  • 使用 @param 标签为构造函数参数添加注释
  • 类注释的第一行会作为 bean 的 comment 属性

${"=".repeat(80)}
`);
}

async function main() {
    const startTime = Date.now();

    // 0. 检查命令行参数
    const args = process.argv.slice(2);

    // 检查是否请求帮助
    if (args.includes('-h') || args.includes('--help')) {
        showHelp();
        process.exit(0);
    }

    console.log("Luban Schema 生成器");
    console.log("=".repeat(50));

    const forceRegenerate = args.includes('--force');

    // 解析路径参数
    let REGISTRATIONS_FILE = DEFAULT_REGISTRATIONS_FILE;
    let INPUT_DIR = null;
    let OUTPUT_FILE = DEFAULT_OUTPUT_FILE;
    let useInputMode = false;

    // 检查 --input 参数（优先）
    const inputIndex = args.indexOf('--input');
    if (inputIndex !== -1 && args[inputIndex + 1]) {
        const customPath = args[inputIndex + 1];
        INPUT_DIR = path.isAbsolute(customPath)
            ? customPath
            : path.join(PROJECT_ROOT, customPath);
        useInputMode = true;
    }

    // 检查 --registrations 参数
    const registrationsIndex = args.indexOf('--registrations');
    if (registrationsIndex !== -1 && args[registrationsIndex + 1]) {
        if (useInputMode) {
            console.warn("  警告: 同时指定了 --input 和 --registrations，将使用 --input 模式");
        } else {
            const customPath = args[registrationsIndex + 1];
            REGISTRATIONS_FILE = path.isAbsolute(customPath)
                ? customPath
                : path.join(PROJECT_ROOT, customPath);
        }
    }

    const outputIndex = args.indexOf('--output');
    if (outputIndex !== -1 && args[outputIndex + 1]) {
        const customPath = args[outputIndex + 1];
        OUTPUT_FILE = path.isAbsolute(customPath)
            ? customPath
            : path.join(PROJECT_ROOT, customPath);
    }

    if (useInputMode) {
        console.log(`  输入目录: ${path.relative(PROJECT_ROOT, INPUT_DIR)}`);
    } else {
        console.log(`  输入文件: ${path.relative(PROJECT_ROOT, REGISTRATIONS_FILE)}`);
    }
    console.log(`  输出文件: ${path.relative(PROJECT_ROOT, OUTPUT_FILE)}`);

    if (forceRegenerate) {
        console.log(`  [强制模式] 跳过缓存检查，强制重新生成所有 bean...`);
    } else {
        console.log(`  [增量模式] 只重新生成变化的 bean...`);
    }

    // 2. 解析 tsconfig
    console.log("\n[1/4] 解析 tsconfig.json...");
    const tsConfig = parseTsConfig();

    // 3. 获取待处理的类列表
    let registrations = [];
    let program;

    if (useInputMode) {
        // --input 模式：扫描目录
        console.log("[2/4] 扫描目录...");
        const tsFiles = scanDirectoryForTsFiles(INPUT_DIR);
        console.log(`  找到 ${tsFiles.length} 个 TypeScript 文件`);

        // 创建 TypeScript 程序
        console.log("[3/4] 解析文件并提取导出的类...");
        program = ts.createProgram({
            rootNames: tsFiles,
            options: tsConfig.options,
        });

        // 从每个文件中提取导出的类和接口
        for (const filePath of tsFiles) {
            const sourceFile = program.getSourceFile(filePath);
            if (sourceFile) {
                const exportedItems = extractExportedClasses(sourceFile, filePath);
                for (const item of exportedItems) {
                    registrations.push({
                        className: item.className,
                        filePath: item.filePath,
                        registeredName: null, // --input 模式下没有 registeredName
                        isInterface: item.isInterface, // 标记是否为接口
                    });
                }
            }
        }

        const classCount = registrations.filter(r => !r.isInterface).length;
        const interfaceCount = registrations.filter(r => r.isInterface).length;
        console.log(`  找到 ${classCount} 个导出的类, ${interfaceCount} 个导出的接口:`);
        for (const reg of registrations) {
            console.log(`    - ${reg.className} (${path.relative(PROJECT_ROOT, reg.filePath)})`);
        }
    } else {
        // --registrations 模式：解析注册文件
        console.log("[2/4] 创建 TypeScript 程序...");
        program = ts.createProgram({
            rootNames: [REGISTRATIONS_FILE],
            options: tsConfig.options,
        });

        console.log("[3/4] 解析 registrations.ts...");
        const registrationsSource = program.getSourceFile(REGISTRATIONS_FILE);
        if (!registrationsSource) {
            console.error(`错误: 无法读取 ${REGISTRATIONS_FILE}`);
            process.exit(1);
        }

        const importAliases = extractImportAliases(registrationsSource);
        registrations = extractRegistrations(registrationsSource, importAliases);
        const importPaths = extractImportPaths(registrationsSource);

        // 为每个注册添加文件路径信息
        for (const reg of registrations) {
            const importPath = importPaths.get(reg.className);
            if (importPath) {
                const resolved = resolveModulePath(importPath, tsConfig);
                reg.importPath = importPath;
                reg.resolvedPath = resolved.path;
                reg.isNpmPackage = resolved.isNpmPackage;
            }
        }

        console.log(`  找到 ${registrations.length} 个注册的类:`);
        for (const reg of registrations) {
            const displayName = reg.registeredName
                ? `${reg.registeredName} (${reg.className})`
                : reg.className;
            console.log(`    - ${displayName}`);
        }
    }

    // 4. 解析现有 XML 中的 bean 定义
    console.log("\n[4/4] 解析现有 bean 定义...");
    const existingBeans = parseExistingXml(OUTPUT_FILE);
    console.log(`  找到 ${existingBeans.size} 个现有 bean 定义`);

    // 5. 提取每个类的字段信息（增量更新）
    console.log("\n[5/5] 提取类字段信息（增量更新）...");
    const classes = [];
    const allErrors = [];  // 收集所有类型错误
    let unchangedCount = 0;
    let updatedCount = 0;

    for (const reg of registrations) {
        // Bean 名称：优先使用 registeredName，否则使用 className
        const beanName = reg.registeredName || reg.className;

        let classFilePath;

        if (useInputMode) {
            // --input 模式：直接使用提供的文件路径
            classFilePath = reg.filePath;
        } else {
            // --registrations 模式：需要解析文件路径
            const importPath = reg.importPath;

            if (!importPath) {
                // 检查类是否在注册文件本身定义
                const registrationsContent = fs.readFileSync(REGISTRATIONS_FILE, "utf-8");
                if (registrationsContent.includes(`class ${reg.className}`) ||
                    registrationsContent.includes(`export class ${reg.className}`)) {
                    // 类在注册文件本身定义，直接使用注册文件路径
                    classFilePath = REGISTRATIONS_FILE;
                } else {
                    console.warn(`  警告: 找不到 ${reg.className} 的导入路径或本地定义，跳过`);
                    continue;
                }
            } else {
                classFilePath = reg.resolvedPath;

                // 对于 npm 包，需要在包内搜索类的定义文件
                if (reg.isNpmPackage) {
                    const resolved = resolveModulePath(importPath, tsConfig);
                    const foundPath = findClassInNpmPackage(resolved.packageName, reg.className);
                    if (foundPath) {
                        classFilePath = foundPath;
                    } else {
                        console.warn(`  警告: 在 ${resolved.packageName} 中找不到类 ${reg.className}`);
                        continue;
                    }
                } else if (classFilePath.endsWith("index.ts") || classFilePath.endsWith("index.tsx")) {
                    // 如果解析到 index.ts，说明是目录重新导出，需要在同目录下搜索类定义
                    const dir = path.dirname(classFilePath);
                    const foundPath = findClassInDirectory(dir, reg.className);
                    if (foundPath) {
                        classFilePath = foundPath;
                    } else {
                        console.warn(`  警告: 在 ${dir} 中找不到类 ${reg.className}`);
                        continue;
                    }
                }
            }
        }

        // 计算当前文件的 hash
        const currentFileHash = computeFileHash(classFilePath);
        const existingBean = existingBeans.get(beanName);

        // 比对 hash，决定是否重新生成
        if (!forceRegenerate && existingBean && existingBean.hash === currentFileHash) {
            // hash 相同，跳过重新生成，复用现有定义
            console.log(`  [缓存] ${beanName}: ${classFilePath}`);
            console.log(`    hash 匹配，跳过重新生成`);

            // 从现有 XML 提取 classInfo（简化版，只需要 name 和 hash）
            const classInfo = {
                name: beanName,
                comment: "",
                fields: [],
                hash: currentFileHash,
                cachedXml: existingBean.xml, // 保存缓存的 XML
            };
            classes.push(classInfo);
            unchangedCount++;
        } else {
            // hash 不同或不存在，重新生成
            const itemType = reg.isInterface ? "接口" : "类";
            console.log(`  [生成] ${beanName} (${itemType}): ${classFilePath}`);

            // 需要将文件添加到程序中
            const fullProgram = ts.createProgram({
                rootNames: [REGISTRATIONS_FILE, classFilePath],
                options: tsConfig.options,
            });

            let extractedInfo, errors;
            if (reg.isInterface) {
                // 提取接口字段
                const result = extractInterfaceFields(classFilePath, reg.className, fullProgram);
                extractedInfo = result.interfaceInfo;
                errors = result.errors;
            } else {
                // 提取类字段
                const result = extractClassFields(classFilePath, reg.className, fullProgram);
                extractedInfo = result.classInfo;
                errors = result.errors;
            }

            // 收集类型错误
            if (errors && errors.length > 0) {
                allErrors.push(...errors);
            }

            if (extractedInfo) {
                // 使用 beanName 作为最终的 bean 名称
                extractedInfo.name = beanName;
                console.log(`    字段: ${extractedInfo.fields.map((f) => f.name).join(", ")}`);
                classes.push(extractedInfo);
                updatedCount++;
            } else {
                console.warn(`    警告: 无法提取类信息`);
            }
        }
    }

    // 6. 检查类型错误
    if (allErrors.length > 0) {
        console.error("\n" + "=".repeat(50));
        console.error("❌ 类型检查失败！发现不支持的类型：");
        console.error("=".repeat(50));

        // 按类分组显示错误
        const errorsByClass = new Map();
        for (const error of allErrors) {
            const match = error.match(/\(([^.]+)\.([^)]+)\)/);
            if (match) {
                const [, className, fieldName] = match;
                if (!errorsByClass.has(className)) {
                    errorsByClass.set(className, []);
                }
                errorsByClass.get(className).push({ fieldName, error });
            }
        }

        for (const [className, errors] of errorsByClass) {
            console.error(`\n类: ${className}`);
            for (const { fieldName, error } of errors) {
                console.error(`  ✗ ${error}`);
            }
        }

        console.error("\n" + "=".repeat(50));
        console.error("解决方法：");
        console.error("  1. 在脚本的 TYPE_MAPPING 中添加类型映射");
        console.error("     位置: src/types/reflect/gen-luban-schema.mjs 第 68 行");
        console.error("  2. 在 Luban XML 中定义对应的 bean");
        console.error("     位置: configs/defines/__beans__.xml");
        console.error("  3. 使用基础类型（string, number, boolean）替代复杂类型");
        console.error("=".repeat(50));

        process.exit(1);
    }

    // 7. 生成 XML 文件
    console.log("\n生成 XML 文件...");
    const xml = generateXml(classes);

    // 确保输出目录存在
    const outputDir = path.dirname(OUTPUT_FILE);
    if (!fs.existsSync(outputDir)) {
        fs.mkdirSync(outputDir, { recursive: true });
    }

    fs.writeFileSync(OUTPUT_FILE, xml, "utf-8");
    console.log(`  输出文件: ${OUTPUT_FILE}`);

    const elapsed = Date.now() - startTime;
    console.log("\n" + "=".repeat(50));
    console.log(`完成! 生成了 ${classes.length} 个 bean 定义`);
    console.log(`  缓存命中: ${unchangedCount} 个`);
    console.log(`  重新生成: ${updatedCount} 个`);
    console.log(`  耗时: ${elapsed}ms`);
}

main().catch((err) => {
    console.error("生成失败:", err);
    process.exit(1);
});