local stdlib = {}

stdlib['+'] = function(_, ...)
    local acc = 0
    for _, v in pairs{...} do
        acc = acc + tonumber(v)
    end
    return acc
end

stdlib['-'] = function(_, a, b)
    return a - b
end

stdlib['*'] = function(_, ...)
    local acc = 0
    for _, v in pairs{...} do
        acc = acc * tonumber(v)
    end
    return acc
end

stdlib['<'] = function(_, a, b)
    return a < b
end

stdlib['>'] = function(_, a, b)
    return a > b
end

stdlib['true'] = true
stdlib['false'] = false

function stdlib.list(_, ...) return {...} end

function stdlib.call(ctx, label, args, locals)
    if type(label) == 'function' then
        return label(ctx, unpack(args))
    else
        local f = stdlib.lookup(ctx, label, locals)
        if type(f) ~= "function" then
            error("Attempt to call non-function "..tostring(label))
        end
        return f(ctx, unpack(args))
    end
end

function stdlib.lookup(ctx, label, locals)
    if ctx[label] then
        return ctx[label]
    elseif locals and locals[label] then
        return locals[label]
    elseif ctx.atoms[label] then
        return ctx.atoms[label]
    elseif ctx.env[label] then
        if type(ctx.env[label]) == 'function' then
            return function(ctx, ...)
                return ctx.env[label](...)
            end
        end
        return ctx.env[label]
    end
    error("Unknown atom "..tostring(label))
end

function stdlib.var(ctx, label, args)
    if type(label) == "number" or type(label) == 'function' then
        return label
    elseif label == 'nil' then
        return nil
    else
        return stdlib.lookup(ctx, label, args)
    end
end

function stdlib.eval(ctx, ast, params)
    if type(ast) == 'string' then -- it's outside of ()s, which means we want to pass it by value
        return stdlib.var(ctx, ast, params)
    elseif type(ast) ~= "table" then -- it's a value
        return ast
    end
    --local frs = require 'frscript'
    --print("AST: ", frs.show(ast))
    local label = ast[1]
    local rest = {select(2, unpack(ast))}

    if not label then error "Null expression" end
    if type(label) == 'table' then error "Attempt to eval a non-function" end
    if label == 'literal' then -- the expression is a string or number literal, return it
        return ast[2]
    end
    if ctx.macros[label] then -- evaluate a macro
        local res = ctx.macros[label](ctx, params, unpack(rest))
        if res then
            return stdlib.eval(ctx, res, params)
        end
        return
    end
    local args = {}
    for i, v in pairs(rest) do
        if type(v) == 'table' and v[1] == 'literal' then -- it's a literal
            args[i] = v[2]
        elseif type(v) == 'table' then -- it's an expression
            args[i] = stdlib.eval(ctx, v, params)
        elseif type(v) == "string" then -- it's an atom
            args[i] = stdlib.var(ctx, v, params)
        else -- no further evaluation needed
            args[i] = v
        end
    end
    return stdlib.call(ctx, label, args, params)
end

function stdlib.reload(ctx)
    local atoms = dofile("stdlib.lua")
    for k, v in pairs(atoms) do
        ctx.atoms[k] = v
    end
    local macros = dofile("stdmacros.lua")
    for k, v in pairs(macros) do
        ctx.macros[k] = v
    end
    return true
end

return stdlib

