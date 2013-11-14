local stdmacros = {}

local frs
function stdmacros.ast(ctx, locals, expr)
    frs = frs or require 'frscript'
    return {'literal', frs.show(expr)}
end

function stdmacros.fn(ctx, locals, name, params, expr)
    ctx.atoms[name] = stdmacros.lambda(ctx, locals, params, expr)
end

function stdmacros.lambda(ctx, locals, params, expr)
    frs = frs or require 'frscript'
    return function(ctx, ...)
        local args = {}
        for i, v in pairs(params) do
            args[v] = ({...})[i]
        end
        if locals then
            frs.fill(args, locals)
        end
        return ctx.atoms.eval(ctx, expr, args)
    end
end

function stdmacros.let(ctx, locals, name, val)
    ctx.atoms[name] = ctx.atoms.eval(ctx, val, locals)
    return ctx.atoms[name]
end

stdmacros['if'] = function(ctx, locals, cond, expr1, expr2)
    cond = ctx.atoms.eval(ctx, cond, locals)
    if cond then
        return expr1
    else
        return expr2
    end
end

return stdmacros

