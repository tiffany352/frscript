local stdlib = {}

stdlib['+'] = function(_, ...)
    local acc = 0
    for _, v in pairs{...} do
        acc = acc + tonumber(v)
    end
    return acc
end

function stdlib.list(_, ...) return {...} end

function stdlib.call(ctx, label, args, locals)
    if locals and locals[label] then
        return locals[label](ctx, unpack(args))
    elseif ctx.atoms[label] then
        return ctx.atoms[label](ctx, unpack(args))
    elseif ctx.env[label] then
        return ctx.env[label](unpack(args))
    else
        error("Unknown atom "..tostring(label))
    end
end

function stdlib.eval(ctx, ast, params)
    if type(ast) ~= "table" then -- it's a lone value
        ast = {ast}
    end
    --local frs = require 'frscript'
    --print("AST: ", frs.show(ast))
    local label = ast[1]
    local rest = {select(2, unpack(ast))}

    if not label then error "Null expression" end
    if label == 'literal' then -- the expression is a string or number literal, return it
        return ast[2]
    end
    if #rest == 0 then -- there are no arguments supplied, so we return its value
        return stdlib.var(ctx, label, params)
    end
    if ctx.macros[label] then -- evaluate a macro
        local res = ctx.macros[label](ctx, unpack(rest))
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
    return stdlib.call(ctx, label, args, locals)
end

function stdlib.var(ctx, label, args)
    if ctx[label] then
        return ctx[label]
    elseif args and args[label] then
        return args[label]
    elseif ctx.atoms[label] then
        return ctx.atoms[label]
    elseif type(label) == "number" then
        return label
    end
    error("Unknown atom "..tostring(label))
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

