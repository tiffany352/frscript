local stdmacros = {}

local frs
function stdmacros.ast(ctx, expr)
    local frs = frs or require 'frscript'
    return {'literal', frs.show(expr)}
end

function stdmacros.fn(ctx, name, args, expr)
    ctx.atoms[name] = function(...)
        stdlib.eval(ctx, expr, args)
    end
    return {}
end

return stdmacros

