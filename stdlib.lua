local stdlib = {}

stdlib['+'] = function(_, ...)
    local acc = 0
    for _, v in pairs{...} do
        acc = acc + tonumber(v)
    end
    return acc
end

function stdlib.list(_, ...) return {...} end

function stdlib.let(ctx, name, val)
    ctx.atoms[name] = val
    return val
end

function stdlib.eval(ctx, ast, args)
    if type(ast) ~= "table" then
        ast = {ast}
    end
    local label = ast[1]
    local rest = {select(2, unpack(ast))}
    if label == 'literal' then
        return stdlib.var(ctx, rest[1], args)
    end
    if ctx.macros[label] then
        return stdlib.eval(ctx, ctx.macros[label](ctx, unpack(rest)))
    end
    local args = {}
    for i, v in pairs(rest) do
        args[i] = type(v) == "table" and stdlib.eval(ctx, v) or v
    end
    --print("eval", label, unpack(rest))
    --print("args", unpack(args))
    if args and args[label] then
        return args[label](ctx, unpack(args))
    elseif ctx.atoms[label] then
        return ctx.atoms[label](ctx, unpack(args))
    elseif ctx.env[label] then
        return ctx.env[label](unpack(args))
    else
        error("Unknown atom "..label)
    end
end

function stdlib.var(ctx, label, args)
    if args and args[label] then
        return args[label]
    elseif ctx.atoms[label] then
        return ctx.atoms[label]
    end
    return label
end

function stdlib.reload(ctx)
    setmetatable(ctx.atoms, {__index=dofile("stdlib.lua")})
    setmetatable(ctx.macros, {__index=dofile("stdmacros.lua")})
    return true
end

return stdlib

